//! The v2 engine, driven through the ordinary C frontend.
//!
//! Everything here builds a binding with the same declarations v1 would take
//! and states the engine explicitly, so a test never depends on what
//! `PREBINDGEN_PIPELINE` happens to hold in the runner's environment.

use prebindgen_registry::pipeline::Pipeline;
use prebindgen_registry_v2::{ElementKind, Outcome};

use super::*;
use crate::test_util::unique_test_dir;

/// A small but complete C binding: one opaque handle, one enum, one callback,
/// one exported function and one acknowledged ignore.
fn binding() -> CbindgenBuilder {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![
        (
            syn::parse_quote!(
                pub struct Calculator {
                    value: f64,
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub enum Operation {
                    Add,
                    Sub,
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn calculator_new() -> Calculator {
                    unimplemented!()
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn calculator_internal(c: &Calculator) -> f64 {
                    unimplemented!()
                }
            ),
            loc.clone(),
        ),
    ]);

    Cbindgen::builder()
        .items(items)
        .source_module(syn::parse_quote!(fixture))
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_function(|name| format!("z_{name}"))
        .opaque_ptr(syn::parse_quote!(Calculator))
        .enum_type(syn::parse_quote!(Operation))
        .callback(syn::parse_quote!(impl Fn(f64) + Send + Sync + 'static))
        .base_name("value")
        .function(syn::parse_quote!(calculator_new))
        .ignore_function(syn::parse_quote!(calculator_internal))
}

/// The gate #719 §A names: the whole declaration set reaches v2, and every
/// requested element comes back accounted for — under the names this adapter's
/// manglers give them, not under a v2 invention.
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
            "type:Calculator",
            "type:Operation",
            "callback:impl Fn(f64)",
            "fn:calculator_internal",
            "fn:calculator_new",
        ],
        "every declaration, sorted deterministically"
    );

    let placements: Vec<&str> = manifest
        .elements
        .iter()
        .map(|entry| entry.element.placement.as_str())
        .collect();
    assert!(
        placements.contains(&"calculator_t") && placements.contains(&"z_calculator_new"),
        "the frontend's manglers name the C surface: {placements:?}"
    );

    let counts = manifest.counts();
    assert_eq!(counts.emitted, 0, "nothing here is a data struct or scalar");
    assert_eq!(counts.skipped, 4);
    assert_eq!(counts.ignored, 1, "an ignore is a decision, not a gap");
}

/// An unimplemented capability is a skip with a code and a path — never a
/// silent omission, and never an error. The code names the declarator the
/// value was declared with, and the path says where the walk stopped.
#[test]
fn a_missing_capability_is_reported_per_element() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let manifest = generated.manifest().expect("v2 produces a manifest");

    let function = manifest
        .elements
        .iter()
        .find(|entry| entry.element.id.as_str() == "fn:calculator_new")
        .expect("the declared function is in the manifest");
    let skip = function
        .outcome
        .skip()
        .expect("an opaque return is not lowered");
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.c.opaque_ptr",
        "a stable code, one per declarator, so the report can be diffed"
    );
    assert_eq!(skip.path(), "fn:calculator_new -> return");

    // Grouped by cause, so one missing capability is stated once with the list
    // of roots it took down: the handle, and the function returning it.
    let groups = manifest.skips_by_capability();
    assert_eq!(groups["unsupported.c.opaque_ptr"].len(), 2);
    // An enum has no fields to walk, so the registry refuses it before the
    // target is asked; the entry's representation still says `enum_type`.
    assert_eq!(groups["unsupported.type.not_a_record"].len(), 1);
    assert_eq!(groups["unsupported.callback.not_implemented"].len(), 1);
}

/// The implemented subset, through the ordinary frontend: a by-value data
/// struct of scalars and a function taking it. Both are emitted, under the
/// manglers' names, and the wrapper reads the struct's members.
#[test]
fn a_data_struct_and_a_function_over_it_are_emitted() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![
        (
            syn::parse_quote!(
                pub struct Stamp {
                    pub secs: i64,
                    pub nanos: i64,
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn stamp_sum(stamp: Stamp) -> i64 {
                    unimplemented!()
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn stamp_new(secs: i64) -> Stamp {
                    unimplemented!()
                }
            ),
            loc,
        ),
    ]);
    let generated = Cbindgen::builder()
        .items(items)
        .source_module(syn::parse_quote!(fixture))
        .mangle_type_name(|base| format!("{base}_t"))
        .mangle_function(|name| format!("z_{name}"))
        .data_struct(syn::parse_quote!(Stamp))
        .function(syn::parse_quote!(stamp_sum))
        .function(syn::parse_quote!(stamp_new))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let manifest = generated.manifest().expect("v2 produces a manifest");
    let counts = manifest.counts();
    assert_eq!((counts.emitted, counts.skipped), (2, 1), "{manifest:?}");

    let dir = unique_test_dir("cbindgen_v2_emitted");
    let path = generated
        .write_rust(dir.join("bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&path).unwrap();
    let compact: String = rust.split_whitespace().collect();
    assert!(
        compact.contains(
            "#[repr(C)]#[allow(non_camel_case_types)]pubstructstamp_t{pubsecs:i64,pubnanos:i64,}"
        ),
        "{rust}"
    );
    assert!(
        compact.contains("pubextern\"C\"fnz_stamp_sum(stamp:stamp_t)->i64{"),
        "{rust}"
    );
    assert!(
        compact.contains("fixture::Stamp{secs:v0,nanos:v1,}"),
        "{rust}"
    );
    assert!(compact.contains("fixture::stamp_sum(v2)"), "{rust}");
    // A record leaving Rust needs a construction the target does not supply
    // yet, so the function returning one is a reported skip, not a wrapper.
    assert!(!rust.contains("z_stamp_new"), "{rust}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// An ignore keeps its meaning under v2 and is counted apart from the gaps.
#[test]
fn an_ignore_is_classified_separately() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let manifest = generated.manifest().expect("v2 produces a manifest");
    let ignored = manifest
        .elements
        .iter()
        .find(|entry| entry.element.id.as_str() == "fn:calculator_internal")
        .expect("the ignore is accounted for");
    assert_eq!(ignored.outcome, Outcome::Ignored);
    assert_eq!(ignored.element.kind, ElementKind::Function);
}

/// A declared function the source never captured is a build error under v2 as
/// it is under v1 — a typo is not a capability question.
#[test]
fn a_declared_function_with_no_captured_item_is_an_error() {
    let error = binding()
        .function(syn::parse_quote!(calculator_typo))
        .build_with(Pipeline::V2)
        .expect_err("a declaration that names nothing is refused");
    let message = error.to_string();
    assert!(message.contains("calculator_typo"), "{message}");
    assert!(
        message.contains("match no captured"),
        "the message says why: {message}"
    );
}

/// The written Rust names the engine that produced it, so an include from the
/// wrong output root says so on its first line.
#[test]
fn the_generated_rust_is_stamped_with_its_pipeline() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let dir = unique_test_dir("cbindgen_v2");
    let path = generated
        .write_rust(dir.join("bindings.rs"))
        .expect("write_rust");
    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(
        contents.starts_with("// Generated by prebindgen v2"),
        "{contents}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// Both renderings of the manifest land beside the generated file.
#[test]
fn the_manifest_is_written_as_json_and_markdown() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    let dir = unique_test_dir("cbindgen_v2_manifest");
    let written = generated.write_manifest(&dir).expect("write_manifest");
    assert_eq!(written.len(), 2);
    let json = std::fs::read_to_string(&written[0]).unwrap();
    assert!(json.contains("\"schema_version\": 1"), "{json}");
    assert!(json.contains("\"pipeline\": \"v2\""), "{json}");
    assert!(json.contains("unsupported.c.opaque_ptr"), "{json}");
    let markdown = std::fs::read_to_string(&written[1]).unwrap();
    assert!(markdown.contains("## Skipped, by cause"), "{markdown}");
    let _ = std::fs::remove_dir_all(&dir);
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
        .function("calculator_new")
        .is_some());
}
