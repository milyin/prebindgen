//! The v2 engine, driven through the ordinary JNI frontend.
//!
//! The declarations are the ones v1 takes — the same `package!`, the same
//! `ptr_class!`, the same `set_*` settings — and the engine is stated
//! explicitly, so a test never depends on the runner's `PREBINDGEN_PIPELINE`.

use prebindgen_registry::pipeline::Pipeline;
use prebindgen_registry_v2::Outcome;

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
