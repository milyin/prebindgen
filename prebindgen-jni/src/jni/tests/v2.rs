//! The v2 engine, driven through the ordinary JNI frontend.
//!
//! The declarations are the ones v1 takes — the same `package!`, the same
//! `ptr_class!`, the same `set_*` settings — and the engine is stated
//! explicitly, so a test never depends on the runner's `PREBINDGEN_PIPELINE`.

use prebindgen_registry::pipeline::Pipeline;
use prebindgen_registry_v2::{ElementKind, Outcome, SourceKind};

use super::*;

/// A small but complete JNI binding: an opaque handle with a method and a
/// factory, a free package function, and an acknowledged ignore.
fn binding() -> JniGenBuilder {
    JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .set_fun_name_mangle(|_package, name| format!("do{}", capitalize(name)))
        .items(fixture_items())
        .package(
            crate::package!("thing")
                .class(
                    crate::ptr_class!(ZThing)
                        .constructor(prebindgen_registry::fun!(z_thing_new))
                        .method(prebindgen_registry::fun!(z_thing_size)),
                )
                .fun(prebindgen_registry::fun!(z_thing_describe)),
        )
        .ignore(prebindgen_registry::fun!(z_thing_internal))
}

fn capitalize(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn fixture_items() -> Vec<(syn::Item, SourceLocation)> {
    let loc = myflat_loc();
    let sources: &[&str] = &[
        "pub fn z_thing_new() -> ZThing { unimplemented!() }",
        "pub fn z_thing_size(t: &ZThing) -> i64 { unimplemented!() }",
        "pub fn z_thing_describe(t: &ZThing) -> i64 { unimplemented!() }",
        "pub fn z_thing_internal(t: &ZThing) -> i64 { unimplemented!() }",
    ];
    declare_referenced(
        sources
            .iter()
            .map(|src| (syn::Item::Fn(syn::parse_str(src).unwrap()), loc.clone()))
            .collect::<Vec<_>>(),
    )
}

/// The gate #719 §A names: the whole declaration set reaches v2, and every
/// requested element comes back accounted for — placed where this adapter's
/// package prefix and name-mangle hooks put it, not where v2 guesses.
#[test]
fn every_declared_element_is_accounted_for() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let manifest = generated.manifest().expect("v2 produces a manifest");

    let ids: Vec<&str> = manifest
        .elements
        .iter()
        .map(|entry| entry.element.id.as_str())
        .collect();
    assert_eq!(
        ids,
        [
            "type:ZThing",
            "fn:z_thing_describe",
            "fn:z_thing_internal",
            "fn:z_thing_new",
            "fn:z_thing_size",
        ]
    );

    let placement = |id: &str| {
        manifest
            .elements
            .iter()
            .find(|entry| entry.element.id.as_str() == id)
            .map(|entry| entry.element.placement.clone())
            .unwrap_or_default()
    };
    assert_eq!(placement("type:ZThing"), "io.test.jni.thing.ZThing");
    assert_eq!(
        placement("fn:z_thing_size"),
        "io.test.jni.thing.ZThing.zThingSize"
    );
    // The `set_fun_name_mangle` closure travelled across intact: a package
    // function is placed under the name it returns.
    assert_eq!(
        placement("fn:z_thing_describe"),
        "io.test.jni.thing.doZThingDescribe"
    );

    let counts = manifest.counts();
    assert_eq!(
        counts.emitted, 0,
        "nothing here is a data class or a scalar"
    );
    assert_eq!(counts.skipped, 4);
    assert_eq!(counts.ignored, 1);

    // Each skip says what the value waits on and where the walk stopped: a
    // borrowed handle is a reference, which no v2 carrier holds yet.
    let skip = |id: &str| {
        manifest
            .elements
            .iter()
            .find(|entry| entry.element.id.as_str() == id)
            .and_then(|entry| entry.outcome.skip())
            .map(|skip| (skip.capability.as_str().to_string(), skip.path()))
            .expect("skipped")
    };
    assert_eq!(
        skip("fn:z_thing_describe"),
        (
            "unsupported.jni.carrier".to_string(),
            "fn:z_thing_describe -> param 0".to_string()
        )
    );
}

/// The implemented subset, through the ordinary frontend: a `data_class!` of
/// scalars and a `fun!` taking it by value. The Rust reads the object's
/// properties through the environment and exports the harness's symbol; the
/// Kotlin declares the class, the native method on the harness, and the
/// function a caller uses — under the package prefix and the function-name
/// hook this binding set.
#[test]
fn a_data_class_and_a_function_over_it_are_emitted() {
    let loc = myflat_loc();
    let sources: &[&str] = &[
        "pub struct Stamp { pub secs: i64, pub nanos: i64 }",
        "pub fn stamp_sum(stamp: Stamp) -> i64 { unimplemented!() }",
        "pub fn stamp_new(secs: i64) -> Stamp { unimplemented!() }",
    ];
    let items = declare_referenced(
        sources
            .iter()
            .map(|src| (syn::parse_str::<syn::Item>(src).unwrap(), loc.clone()))
            .collect::<Vec<_>>(),
    );
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .set_jni_native_init("io.test.jni.Lib.load()")
        .items(items)
        .package(
            crate::package!()
                .class(crate::data_class!(Stamp).name("Timestamp"))
                .fun(prebindgen_registry::fun!(stamp_sum))
                .fun(prebindgen_registry::fun!(stamp_new)),
        )
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let manifest = generated.manifest().expect("v2 produces a manifest");
    let counts = manifest.counts();
    assert_eq!((counts.emitted, counts.skipped), (2, 1), "{manifest:?}");

    let dir = unique_test_dir("jnigen_v2_emitted");
    let _ = std::fs::remove_dir_all(&dir);
    let rust = generated
        .write_rust(dir.join("generated_bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust).unwrap();
    let compact: String = rust.split_whitespace().collect();
    assert!(
        compact.contains(
            "pubextern\"system\"fnJava_io_test_jni_JNINative_stampSum(mutenv:jni::JNIEnv<'_>,\
             _this:jni::objects::JObject<'_>,stamp:jni::objects::JObject<'_>,)->jni::sys::jlong{"
        ),
        "{rust}"
    );
    assert!(
        compact.contains("env.call_method(&stamp,\"getSecs\",\"()J\",&[])"),
        "{rust}"
    );
    assert!(
        compact.contains("myflat::Stamp{secs:v0,nanos:v1,}"),
        "{rust}"
    );
    assert!(
        compact.contains("report_jni_error(&mutenv,error)"),
        "{rust}"
    );
    assert!(
        !rust.contains("stampNew"),
        "a record leaving Rust is a skip: {rust}"
    );

    let written = generated
        .write_kotlin(&dir.join("kotlin"))
        .expect("write_kotlin");
    assert_eq!(written.len(), 1, "{written:?}");
    let kotlin = std::fs::read_to_string(&written[0]).unwrap();
    for line in [
        "package io.test.jni",
        "public data class Timestamp(val secs: Long, val nanos: Long)",
        "public fun stampSum(stamp: Timestamp): Long = JNINative.stampSum(stamp)",
        "internal object JNINative {",
        "io.test.jni.Lib.load()",
        "external fun stampSum(stamp: Timestamp): Long",
    ] {
        assert!(
            kotlin.lines().any(|emitted| emitted.trim() == line),
            "missing `{line}`:\n{kotlin}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// Class members are separately selected: the class and each of its methods are
/// elements in their own right, so one can be skipped without the others.
#[test]
fn class_members_are_elements_of_their_own() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let manifest = generated.manifest().expect("v2 produces a manifest");
    let representation = |id: &str| {
        manifest
            .elements
            .iter()
            .find(|entry| entry.element.id.as_str() == id)
            .map(|entry| entry.element.representation.clone())
            .unwrap_or_default()
    };
    assert_eq!(representation("type:ZThing"), "ptr_class");
    assert_eq!(representation("fn:z_thing_new"), "constructor");
    assert_eq!(representation("fn:z_thing_size"), "method");
    assert_eq!(representation("fn:z_thing_describe"), "fun");
}

/// An ignore keeps its meaning under v2 and is counted apart from the gaps.
#[test]
fn an_ignore_is_classified_separately() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let manifest = generated.manifest().expect("v2 produces a manifest");
    let ignored = manifest
        .elements
        .iter()
        .find(|entry| entry.element.id.as_str() == "fn:z_thing_internal")
        .expect("the ignore is accounted for");
    assert_eq!(ignored.outcome, Outcome::Ignored);
}

/// Both writers work under an engine that generated nothing. A zero-output plan
/// is still a plan, and a build script calls `write_rust` and `write_kotlin`
/// without asking which engine ran.
#[test]
fn the_ordinary_writers_run_under_v2() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let dir = unique_test_dir("jnigen_v2_writers");
    let _ = std::fs::remove_dir_all(&dir);

    let rust = generated
        .write_rust(dir.join("generated_bindings.rs"))
        .expect("write_rust");
    let contents = std::fs::read_to_string(&rust).unwrap();
    assert!(
        contents.starts_with("// Generated by prebindgen v2"),
        "the file names the engine that produced it: {contents}"
    );

    // Nothing here is lowered, so no Kotlin is written — and the root still
    // exists, so a Gradle source set pointed at it resolves to an empty set
    // rather than a missing directory.
    let kotlin_root = dir.join("kotlin");
    assert!(generated
        .write_kotlin(&kotlin_root)
        .expect("write_kotlin")
        .is_empty());
    assert!(kotlin_root.is_dir());

    let written = generated.write_manifest(&dir).expect("write_manifest");
    assert_eq!(written.len(), 2);
    let json = std::fs::read_to_string(&written[0]).unwrap();
    assert!(json.contains("\"pipeline\": \"v2\""), "{json}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A binding-local fn is declared like any other, at a class member and at a
/// package function, and neither requires a captured item. Both were refused as
/// missing declarations before, while v1 built them.
#[test]
fn a_binding_local_fn_is_placed_without_being_captured() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(fixture_items())
        .package(
            crate::package!("thing")
                .class(
                    crate::ptr_class!(ZThing).method(
                        prebindgen_registry::fun!(crate::local_size)
                            .sig(prebindgen_registry::sig!((t: &ZThing) -> i64)),
                    ),
                )
                .fun(
                    prebindgen_registry::fun!(crate::local_tag)
                        .sig(prebindgen_registry::sig!(() -> i64)),
                ),
        )
        .build_with(Pipeline::V2)
        .expect("a binding-local fn needs no captured item");
    let manifest = generated.manifest().expect("v2 produces a manifest");

    // Once each: a helper is stated where it is bound, and a second entry for
    // the helper itself would give one id to two elements.
    let ids: Vec<&str> = manifest
        .elements
        .iter()
        .map(|entry| entry.element.id.as_str())
        .collect();
    assert_eq!(ids, ["type:ZThing", "fn:local_size", "fn:local_tag"]);
}

/// `constant!(X).fun(fun!(f))` surfaces a Kotlin `val` backed by a nullary
/// **function**. The captured item is a function, and looking `f` up among the
/// constants reported a typo that was not there.
#[test]
fn a_function_backed_constant_resolves_against_the_function() {
    let generated = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .items(fixture_items())
        .package(
            crate::package!("thing")
                .class(crate::ptr_class!(ZThing))
                .constant(
                    crate::constant!(THE_SIZE).fun(prebindgen_registry::fun!(z_thing_describe)),
                ),
        )
        .build_with(Pipeline::V2)
        .expect("a function-backed constant resolves");
    let manifest = generated.manifest().expect("v2 produces a manifest");
    let constant = manifest
        .elements
        .iter()
        .find(|entry| entry.element.representation == "constant_fun")
        .expect("the constant is accounted for");
    assert_eq!(constant.element.rust_origin, "z_thing_describe");
    assert_eq!(constant.element.kind, ElementKind::Const);
    // The target gets a `val`; the source must hold a function.
    assert_eq!(constant.element.source, SourceKind::Function);
}

/// Selecting v1 explicitly still runs v1: the same declarations, the whole
/// existing surface, and no manifest.
#[test]
fn v1_is_unchanged_and_reachable_by_name() {
    let generated = binding().build_with(Pipeline::V1).expect("v1 resolves");
    assert_eq!(generated.pipeline(), Pipeline::V1);
    assert!(generated.manifest().is_none());
    assert!(generated
        .registry()
        .flat()
        .function("z_thing_new")
        .is_some());
}
