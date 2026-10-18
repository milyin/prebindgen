//! Generation-contract tests use only flat, without the preparation crate.
use super::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

struct Input(usize);
impl InputConversion for Input {
    type Wire = &'static str;
    fn emit(&self) -> Result<InputPlan<Self::Wire>> {
        let index = self.0;
        let name = format_ident!("arg{index}");
        let validate = format!("validate{index}");
        let acquire = format!("acquire{index}");
        let construct = format!("construct{index}");
        Ok(InputPlan {
            requirements: vec![CallRequirement {
                key: "test.ack".into(),
                parameters: quote!(),
                acknowledge: quote!(log("acknowledge"); if fail_acknowledge { return Err("acknowledgement".into()); }),
            }],
            representation: None,
            wires: vec![],
            validation: quote!(log(#validate); if #index == 1 && fail_validation { return Err("validation".into()); }),
            resources: vec![],
            acquire: quote!(log(#acquire);),
            statements: quote!(log(#construct); let #name = 1;),
            argument: quote!(#name),
        })
    }
}
struct Output;
impl OutputConversion for Output {
    type Wire = &'static str;
    fn emit(&self) -> Result<OutputPlan<Self::Wire>> {
        Ok(OutputPlan {
            requirements: Vec::new(),
            delivery: Delivery {
                statements: quote!(log("deliver"); if fail_delivery { return Err("delivery".into()); }),
                result: source_result_expr(),
            },
            representation: None,
            wires: vec![Wire {
                component: ComponentPath::default(),
                name: syn::parse_quote!(out),
                kind: "int",
                active: Condition::default(),
            }],
            statements: quote!(log("prepare");),
            values: vec![source_result_expr()],
            commit: quote!(log("commit");),
        })
    }
}

#[test]
fn generated_call_enforces_order_across_validation_and_delivery_failures() {
    let items = syn::parse_file("pub fn add(a:i32,b:i32)->i32 { log(\"source\"); a+b }").unwrap();
    let mut flat = Flat::<Input, Output>::parse(
        items
            .items
            .clone()
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap();
    let f = flat.function_mut("add").unwrap();
    f.params[0].ty.slot = Some(Input(0));
    f.params[1].ty.slot = Some(Input(1));
    f.ret.slot = Some(Output);
    let call = prepare_call(&flat, "add").unwrap();
    assert_eq!(call.requirements().len(), 1);
    let body = call.emit();
    let code = quote! {
        use std::sync::Mutex;
        static LOG: Mutex<Vec<&'static str>> = Mutex::new(Vec::new());
        fn log(event: &'static str) { LOG.lock().unwrap().push(event); }
        #items
        fn run(fail_validation:bool, fail_acknowledge:bool, fail_delivery:bool) -> Result<i32,String> {
            (|| { #body })()
        }
        fn main() {
            assert_eq!(run(true, false, false), Err("validation".into()));
            assert_eq!(*LOG.lock().unwrap(), ["validate0", "validate1"]);
            LOG.lock().unwrap().clear();
            assert_eq!(run(false, true, false), Err("acknowledgement".into()));
            assert_eq!(*LOG.lock().unwrap(), ["validate0", "validate1", "acknowledge"]);
            LOG.lock().unwrap().clear();
            assert_eq!(run(false, false, true), Err("delivery".into()));
            assert_eq!(*LOG.lock().unwrap(), ["validate0", "validate1", "acknowledge", "acquire0", "acquire1", "construct0", "construct1", "source", "prepare", "deliver"]);
            LOG.lock().unwrap().clear();
            assert_eq!(run(false, false, false), Ok(2));
            assert_eq!(*LOG.lock().unwrap(), ["validate0", "validate1", "acknowledge", "acquire0", "acquire1", "construct0", "construct1", "source", "prepare", "deliver", "commit"]);
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("protocol.rs");
    let executable = dir.path().join("protocol");
    std::fs::write(&path, code.to_string()).unwrap();
    let compiled = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&path)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let run = std::process::Command::new(executable).output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

fn record_model() -> Flat<InputPlan<()>, OutputPlan<&'static str>, String> {
    let items =
        syn::parse_file("pub struct Point { pub x:f64, pub y:f64 } pub fn point()->Point {}")
            .unwrap();
    let mut flat = Flat::parse(items.items.into_iter().map(|i| (i, Default::default()))).unwrap();
    let id = RepresentationId("PointFields".into());
    flat.representations
        .insert(
            id.clone(),
            Representation {
                source_type: syn::parse_quote!(crate::Point),
                components: ["x", "y"]
                    .into_iter()
                    .map(|name| Component {
                        path: ComponentPath(vec![ComponentSegment::Field(name.into())]),
                        name: name.into(),
                        source_fields: Some(vec![name.into()]),
                        representation: None,
                    })
                    .collect(),
                data: "Point".into(),
            },
        )
        .unwrap();
    flat.representations.set_canonical(&id).unwrap();
    let result = source_result();
    flat.function_mut("point").unwrap().ret.slot = Some(OutputPlan {
        requirements: Vec::new(),
        delivery: Default::default(),
        representation: Some(id),
        // Deliberately reverse wire order: mapping must use component identities.
        wires: ["y", "x"]
            .into_iter()
            .map(|name| Wire {
                component: ComponentPath(vec![ComponentSegment::Field(name.into())]),
                name: format_ident!("wire_{name}"),
                kind: "Double",
                active: Condition::default(),
            })
            .collect(),
        statements: TokenStream::new(),
        values: vec![quote!(#result.y), quote!(#result.x)],
        commit: TokenStream::new(),
    });
    flat
}

#[test]
fn type_declarations_and_values_share_explicit_component_identities() {
    let flat = record_model();
    let call = prepare_call(&flat, "point").unwrap();
    let schema = flat
        .representations
        .get(call.output_representation().unwrap())
        .unwrap();
    let assignments: Vec<_> = call
        .outputs()
        .map(|out| {
            let field = schema
                .components
                .iter()
                .find(|c| c.path == out.wire.component)
                .unwrap();
            format!("{} = {}", field.name, out.value)
        })
        .collect();
    assert_eq!(
        format!("{}({})", schema.data, assignments.join(", ")),
        "Point(y = __result . y, x = __result . x)"
    );
    assert_eq!(
        flat.representations
            .canonical(&syn::parse_quote!(crate::Point)),
        call.output_representation()
    );
}

#[test]
fn generation_rejects_invalid_mappings_and_output_arity() {
    let mut flat = record_model();
    flat.function_mut("point")
        .unwrap()
        .ret
        .slot
        .as_mut()
        .unwrap()
        .wires[0]
        .component = ComponentPath(vec![ComponentSegment::Named("absent".into())]);
    assert!(prepare_call(&flat, "point")
        .err()
        .unwrap()
        .contains("no component"));
    let mut flat = record_model();
    flat.function_mut("point")
        .unwrap()
        .ret
        .slot
        .as_mut()
        .unwrap()
        .values
        .pop();
    assert!(prepare_call(&flat, "point")
        .err()
        .unwrap()
        .contains("count differs"));
    let mut flat = record_model();
    let id = RepresentationId("PointFields".into());
    let mut shape = flat.representations.get(&id).unwrap().clone();
    shape.components[0].source_fields = Some(vec!["absent".into()]);
    flat.representations.insert(id, shape).unwrap();
    assert!(flat
        .validate_representations()
        .unwrap_err()
        .contains("unknown source field"));
}

#[test]
fn changing_canonical_default_does_not_reinterpret_bound_occurrences() {
    let mut flat = record_model();
    let handle = RepresentationId("PointHandle".into());
    flat.representations
        .insert(
            handle.clone(),
            Representation {
                source_type: syn::parse_quote!(crate::Point),
                components: vec![],
                data: "Handle".into(),
            },
        )
        .unwrap();
    flat.representations.set_canonical(&handle).unwrap();
    let call = prepare_call(&flat, "point").unwrap();
    assert_eq!(
        call.output_representation(),
        Some(&RepresentationId("PointFields".into()))
    );
    assert_eq!(call.outputs().len(), 2);
}

#[test]
fn standalone_type_schemas_validate_without_executable_slots() {
    let mut flat = record_model();
    flat.functions.clear();
    flat.validate_representations().unwrap();
    let id = RepresentationId("PointFields".into());
    let mut shape = flat.representations.get(&id).unwrap().clone();
    shape.components[0].representation = Some(RepresentationId("MissingScalar".into()));
    flat.representations.insert(id, shape).unwrap();
    assert!(flat
        .validate_representations()
        .unwrap_err()
        .contains("unknown representation"));
}

#[test]
fn a_selected_shape_requires_values_for_all_components() {
    let mut flat = record_model();
    let output = flat
        .function_mut("point")
        .unwrap()
        .ret
        .slot
        .as_mut()
        .unwrap();
    output.wires.pop();
    output.values.pop();
    assert!(prepare_call(&flat, "point")
        .err()
        .unwrap()
        .contains("has no wire"));
}

#[test]
fn several_physical_wires_may_supply_one_component() {
    let mut flat = record_model();
    let output = flat
        .function_mut("point")
        .unwrap()
        .ret
        .slot
        .as_mut()
        .unwrap();
    let mut extra = output.wires[0].clone();
    extra.name = syn::parse_quote!(additional_wire);
    output.wires.push(extra);
    output.values.push(quote!(0));
    flat.validate_representations().unwrap();
    assert_eq!(prepare_call(&flat, "point").unwrap().outputs().len(), 3);
}

#[test]
fn shared_requirements_merge_across_slots_and_reject_conflicts() {
    let items = syn::parse_file("pub fn add(a:i32,b:i32)->i32 { a+b }").unwrap();
    let mut flat = Flat::<InputPlan<&str>, OutputPlan<&str>>::parse(
        items.items.into_iter().map(|i| (i, Default::default())),
    )
    .unwrap();
    let first = Input(0).emit().unwrap();
    let mut second = Input(1).emit().unwrap();
    let mut output = Output.emit().unwrap();
    output.requirements = first.requirements.clone();
    output.requirements.push(CallRequirement {
        key: "test.result".into(),
        parameters: quote!(out: *mut i32),
        acknowledge: quote!(),
    });
    let f = flat.function_mut("add").unwrap();
    f.params[0].ty.slot = Some(first);
    f.params[1].ty.slot = Some(second.clone());
    f.ret.slot = Some(output);
    let call = prepare_call(&flat, "add").unwrap();
    assert_eq!(
        call.requirements()
            .iter()
            .map(|r| r.key.as_str())
            .collect::<Vec<_>>(),
        ["test.ack", "test.result"]
    );
    assert_eq!(
        call.requirements()[1].parameters.to_string(),
        quote!(out: *mut i32).to_string()
    );
    for conflict in [true, false] {
        let mut bad = second.clone();
        if conflict {
            bad.requirements[0].parameters = quote!(other: i32);
        } else {
            bad.requirements[0].acknowledge = quote!(different_ack());
        }
        flat.function_mut("add").unwrap().params[1].ty.slot = Some(bad);
        assert!(prepare_call(&flat, "add")
            .err()
            .unwrap()
            .contains("conflicting call requirement `test.ack`"));
    }
    second.requirements[0].key = "test.another".into();
    flat.function_mut("add").unwrap().params[1].ty.slot = Some(second);
    assert_eq!(prepare_call(&flat, "add").unwrap().requirements().len(), 3);
}
