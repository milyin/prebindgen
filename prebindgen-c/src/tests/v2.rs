//! The v2 engine, driven through the ordinary C frontend.
//!
//! Everything here builds a binding with the same declarations v1 would take
//! and states the engine explicitly, so a test never depends on what
//! `PREBINDGEN_PIPELINE` happens to hold in the runner's environment.

use prebindgen_registry::pipeline::Pipeline;

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
/// requested declaration comes back either generated or skipped.
#[test]
fn every_declared_element_is_accounted_for() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");

    let skipped: Vec<String> = generated
        .skipped()
        .iter()
        .map(|(declaration, _)| declaration.to_string())
        .collect();
    assert_eq!(skipped, ["callback:impl Fn(f64)"]);

    // The handle — a struct carried whole under `opaque_ptr`, its fields never
    // read — the enum, and the function returning the handle are what is left,
    // under the names this adapter's manglers give them.
    let dir = unique_test_dir("cbindgen_v2_accounted");
    let path = generated
        .write_rust(dir.join("bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(rust.contains("pub struct calculator_t {"), "{rust}");
    assert!(
        rust.contains("pub extern \"C\" fn z_calculator_new()"),
        "{rust}"
    );
}

/// An unimplemented capability is a skip with a code and a path — never a
/// silent omission, and never an error. The code names the declarator the
/// value was declared with, and the path says where the walk stopped.
#[test]
fn a_missing_capability_is_reported_per_element() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");

    // A callback has no lowering at all.
    let codes: Vec<&str> = generated
        .skipped()
        .iter()
        .map(|(_, skip)| skip.capability.as_str())
        .collect();
    assert_eq!(codes, ["unsupported.callback.not_implemented"]);

    // The handle is emitted under the manglers' names — the incomplete type,
    // its destructor, and the function returning one — with the struct's
    // fields left unread.
    let dir = unique_test_dir("cbindgen_v2_handle");
    let path = generated
        .write_rust(dir.join("bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    for expected in [
        "pub struct calculator_t {",
        "pub extern \"C\" fn calculator_drop(this_: *mut calculator_t)",
        "pub extern \"C\" fn z_calculator_new() -> *mut calculator_t",
        "Box::into_raw(Box::new(v0)) as *mut calculator_t",
    ] {
        assert!(rust.contains(expected), "missing `{expected}`:\n{rust}");
    }
    assert!(!rust.contains("value"), "the field is never read:\n{rust}");
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
    assert_eq!(generated.skipped().len(), 1, "{:?}", generated.skipped());

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
    // A struct leaving Rust needs a construction the target does not supply
    // yet, so the function returning one is a reported skip, not a wrapper.
    assert!(!rust.contains("z_stamp_new"), "{rust}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// An ignore silences v1's undeclared-item warning and nothing else: under v2
/// the item is not declared, so nothing is generated for it and nothing
/// accounts for it, and it stays in the model, where a declared item may still
/// depend on it.
#[test]
fn an_ignore_does_not_reach_the_engine() {
    let generated = binding().build_with(Pipeline::V2).expect("v2 plans");
    assert!(
        !generated
            .skipped()
            .iter()
            .any(|(declaration, _)| declaration.to_string() == "fn:calculator_internal"),
        "{:?}",
        generated.skipped()
    );
    let dir = unique_test_dir("cbindgen_v2_ignored");
    let path = generated
        .write_rust(dir.join("bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    assert!(!rust.contains("calculator_internal"), "{rust}");
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

/// An `unsafe fn` is refused: the wrapper is a safe function and the call it
/// renders is a plain one, and hiding the contract in an `unsafe` block would
/// not establish it. Under v1 the wrapper itself is `unsafe`, so the same
/// declaration builds there.
#[test]
fn an_unsafe_source_function_is_a_reported_skip() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![(
        syn::parse_quote!(
            pub unsafe fn raw_sum(a: i64, b: i64) -> i64 {
                unimplemented!()
            }
        ),
        loc,
    )]);
    let generated = Cbindgen::builder()
        .items(items)
        .source_module(syn::parse_quote!(fixture))
        .function(syn::parse_quote!(raw_sum))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one declaration, one skip: {:?}", generated.skipped());
    };
    assert_eq!(declaration.to_string(), "fn:raw_sum");
    assert_eq!(skip.capability.as_str(), "unsupported.fn.unsafe");
    assert_eq!(skip.path(), "fn:raw_sum");
}

/// Selecting v1 explicitly still runs v1: the same declarations, the whole
/// existing surface, and nothing left out.
#[test]
fn v1_is_unchanged_and_reachable_by_name() {
    let generated = binding().build_with(Pipeline::V1).expect("v1 resolves");
    assert_eq!(generated.pipeline(), Pipeline::V1);
    assert!(generated.skipped().is_empty());
    assert!(generated
        .registry()
        .flat()
        .function("calculator_new")
        .is_some());
}

/// A fieldless enum leaves Rust as the C enum this adapter declares for it —
/// the same values under the same names, carrying the numbers Rust assigns —
/// and comes back as a C `int`.
///
/// The wrapper goes between the source type and the carrier by matching one
/// value at a time. Out of Rust the match names every value and cannot fail;
/// if the two enums ever drift apart, the generated Rust stops compiling
/// rather than mapping a value to the wrong one. Into Rust the carrier is an
/// `int`, because C lets an enum variable hold any `int` and a Rust enum
/// holding a number none of its values has is undefined behaviour — so a
/// number no value has fails instead.
#[test]
fn a_fieldless_enum_crosses_as_the_c_enum_declared_for_it() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![
        (
            syn::parse_quote!(
                pub enum Operation {
                    Add,
                    Mul = 7,
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn operation_flip(op: Operation) -> Operation {
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
        .enum_type(syn::parse_quote!(Operation))
        .function(syn::parse_quote!(operation_flip))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert!(generated.skipped().is_empty(), "{:?}", generated.skipped());

    let dir = unique_test_dir("cbindgen_v2_enum");
    let path = generated
        .write_rust(dir.join("bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let compact: String = rust.split_whitespace().collect();

    // The C enum, with the numbers Rust assigns — `Mul` says 7 because the
    // source does, and `Add` says 0 because Rust counts from there.
    assert!(
        compact.contains("pubenumoperation_t{Add=0,Mul=7,}"),
        "{rust}"
    );
    // The wrapper takes an `int` and returns the enum.
    assert!(
        compact.contains("pubextern\"C\"fnoperation_flip(op:::core::ffi::c_int)->operation_t{"),
        "{rust}"
    );
    // One arm per value each way; into Rust, a default arm for a number no
    // value has.
    assert!(
        compact.contains(
            "matchop{0=>::core::result::Result::Ok(fixture::Operation::Add),\
             7=>::core::result::Result::Ok(fixture::Operation::Mul),\
             other=>{::core::result::Result::Err(\
             ::std::format!(\"`operation_t`hasnovaluenumbered{}\",other),)}}"
        ),
        "{rust}"
    );
    assert!(
        compact.contains(
            "matchv1{fixture::Operation::Add=>operation_t::Add,\
             fixture::Operation::Mul=>operation_t::Mul,}"
        ),
        "{rust}"
    );
}

/// A fieldless value keeps the delimiters the source wrote.
///
/// `enum Operation { Add(), Mul {} }` carries nothing, so the model calls it a
/// fieldless enum — but `Add` and `Mul` are not unit variants, and a pattern
/// or a constructor naming them without their delimiters does not compile
/// (E0532/E0533).
///
/// Only the source side of the conversion is spelled that way. The C enum is
/// this target's own declaration and its values are numbers, which is what a
/// C enum has; a variant with a discriminant cannot carry delimiters anyway.
#[test]
fn a_fieldless_value_keeps_its_constructor_shape() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![
        (
            syn::parse_quote!(
                pub enum Operation {
                    Add(),
                    Mul {},
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub fn operation_flip(op: Operation) -> Operation {
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
        .enum_type(syn::parse_quote!(Operation))
        .function(syn::parse_quote!(operation_flip))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    assert!(generated.skipped().is_empty(), "{:?}", generated.skipped());

    let dir = unique_test_dir("cbindgen_v2_enum_shape");
    let path = generated
        .write_rust(dir.join("bindings.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let compact: String = rust.split_whitespace().collect();

    // The mirror is numbers; every mention of the source enum keeps its
    // delimiters, in a pattern and in a constructor alike.
    assert!(
        compact.contains("pubenumoperation_t{Add=0,Mul=1,}"),
        "{rust}"
    );
    assert!(
        compact.contains("0=>::core::result::Result::Ok(fixture::Operation::Add())"),
        "{rust}"
    );
    assert!(
        compact.contains("1=>::core::result::Result::Ok(fixture::Operation::Mul{})"),
        "{rust}"
    );
    assert!(
        compact.contains("fixture::Operation::Add()=>operation_t::Add"),
        "{rust}"
    );
    assert!(
        compact.contains("fixture::Operation::Mul{}=>operation_t::Mul"),
        "{rust}"
    );
}

/// A value written under a `#[cfg]` is refused, rather than mirrored as if it
/// were always there.
///
/// The model numbers every value as present, so a conditional value followed
/// by an implicit one gives numbers the compiled enum disagrees with — and a
/// mirror entry for a value the source crate compiled out names a variant that
/// does not exist, which the generated matches would then reference.
#[test]
fn a_conditional_value_refuses_the_enum() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![(
        syn::parse_quote!(
            pub enum Operation {
                Add,
                #[cfg(any())]
                Mul = 7,
            }
        ),
        loc,
    )]);
    let generated = Cbindgen::builder()
        .items(items)
        .source_module(syn::parse_quote!(fixture))
        .mangle_type_name(|base| format!("{base}_t"))
        .enum_type(syn::parse_quote!(Operation))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one declaration, one skip: {:?}", generated.skipped());
    };
    assert_eq!(declaration.to_string(), "type:Operation");
    assert_eq!(skip.capability.as_str(), "unsupported.c.conditional_value");
}

/// A `#[non_exhaustive]` enum is refused.
///
/// Rust requires a wildcard arm wherever another crate matches such an enum,
/// and a binding crate is always another crate — so a match naming every value
/// the source declares today is still E0004 there. Going out of Rust there is
/// nothing for that arm to produce, so the enum is refused rather than carried
/// with an answer invented for it.
#[test]
fn a_non_exhaustive_enum_is_refused() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![(
        syn::parse_quote!(
            #[non_exhaustive]
            pub enum Operation {
                Add,
                Mul = 7,
            }
        ),
        loc,
    )]);
    let generated = Cbindgen::builder()
        .items(items)
        .source_module(syn::parse_quote!(fixture))
        .mangle_type_name(|base| format!("{base}_t"))
        .enum_type(syn::parse_quote!(Operation))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one declaration, one skip: {:?}", generated.skipped());
    };
    assert_eq!(declaration.to_string(), "type:Operation");
    assert_eq!(
        skip.capability.as_str(),
        "unsupported.c.non_exhaustive_enum"
    );
}

/// A number the model could not evaluate refuses the enum.
///
/// A mirror is the numbers, and the model reads them from literals alone: an
/// arithmetic discriminant, or one naming a `const`, ends the chain and leaves
/// the values after it unnumbered. Emitting the mirror from what is left would
/// renumber them behind the source's back.
#[test]
fn an_unevaluable_number_refuses_the_enum() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![(
        syn::parse_quote!(
            pub enum Operation {
                Add = 1 + 1,
                Mul,
            }
        ),
        loc,
    )]);
    let generated = Cbindgen::builder()
        .items(items)
        .source_module(syn::parse_quote!(fixture))
        .mangle_type_name(|base| format!("{base}_t"))
        .enum_type(syn::parse_quote!(Operation))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one declaration, one skip: {:?}", generated.skipped());
    };
    assert_eq!(declaration.to_string(), "type:Operation");
    assert_eq!(skip.capability.as_str(), "unsupported.c.enum_discriminant");
}

/// An enum with no values refuses too: there is nothing to mirror, and C has
/// no empty enumeration to mirror it as.
#[test]
fn an_enum_with_no_values_is_refused() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![(
        syn::parse_quote!(
            pub enum Operation {}
        ),
        loc,
    )]);
    let generated = Cbindgen::builder()
        .items(items)
        .source_module(syn::parse_quote!(fixture))
        .mangle_type_name(|base| format!("{base}_t"))
        .enum_type(syn::parse_quote!(Operation))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one declaration, one skip: {:?}", generated.skipped());
    };
    assert_eq!(declaration.to_string(), "type:Operation");
    assert_eq!(skip.capability.as_str(), "unsupported.c.empty_enum");
}

/// A number outside a C `int` refuses the enum.
///
/// `#[repr(i64)]` makes `5_000_000_000` valid Rust, but the C enum and the
/// `int` it comes back as are 32 bits, so neither could carry it.
#[test]
fn a_number_outside_int_is_refused() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = declare_referenced(vec![(
        syn::parse_quote!(
            #[repr(i64)]
            pub enum Operation {
                Add = 5_000_000_000,
            }
        ),
        loc,
    )]);
    let generated = Cbindgen::builder()
        .items(items)
        .source_module(syn::parse_quote!(fixture))
        .mangle_type_name(|base| format!("{base}_t"))
        .enum_type(syn::parse_quote!(Operation))
        .build_with(Pipeline::V2)
        .expect("v2 plans");
    let [(declaration, skip)] = generated.skipped() else {
        panic!("one declaration, one skip: {:?}", generated.skipped());
    };
    assert_eq!(declaration.to_string(), "type:Operation");
    assert_eq!(skip.capability.as_str(), "unsupported.c.enum_range");
}
