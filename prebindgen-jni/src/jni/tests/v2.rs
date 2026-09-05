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
    assert_eq!(counts.emitted, 0, "no Kotlin lowering is implemented yet");
    assert_eq!(counts.skipped, 4);
    assert_eq!(counts.ignored, 1);
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

    // No Kotlin is generated yet, and the root still exists — a Gradle source
    // set pointed at it resolves to an empty set, not to a missing directory.
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
