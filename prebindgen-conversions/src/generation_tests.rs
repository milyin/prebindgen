//! Execution tests for the optional staged protocol, without declarations.
use super::*;
use crate::staged::emit_call;
use prebindgen_flat::Result;
use prebindgen_flat::{Conversion, Wire, WireKind};
type Flat<W = syn::Type, C = ()> = prebindgen_flat::Flat<crate::PlanSchema<W, C>>;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

struct Input(usize);
impl Input {
    fn plan(&self) -> InputPlan<syn::Type> {
        let index = self.0;
        let name = format_ident!("arg{index}");
        let validate = format!("validate{index}");
        let acquire = format!("acquire{index}");
        let construct = format!("construct{index}");
        InputPlan {
            fallible: false,
            context: Default::default(),
            acknowledgement: Some(
                quote!(log("acknowledge"); if fail_acknowledge { return Err("acknowledgement".into()); }),
            ),
            node: Default::default(),
            validation: quote!(log(#validate); if #index == 1 && fail_validation { return Err("validation".into()); }),
            resources: vec![],
            acquire: quote!(log(#acquire);),
            statements: quote!(log(#construct); let #name = 1;),
            argument: quote!(#name),
        }
    }
}
struct Output;
impl Output {
    fn plan(&self) -> OutputPlan<syn::Type> {
        OutputPlan {
            fallible: false,
            context: Default::default(),
            delivery: Delivery {
                statements: quote!(log("deliver"); if fail_delivery { return Err("delivery".into()); }),
                result: source_result_expr(),
            },
            outcomes: prebindgen_flat::Outcomes {
                success: prebindgen_flat::WireNode::Leaf(vec![Wire {
                    name: syn::parse_quote!(out),
                    kind: syn::parse_quote!(i32),
                }]),
                source_error: None,
                binding_failure: None,
            },
            statements: quote!(log("prepare");),
            values: vec![(syn::parse_quote!(out), source_result_expr())],
            commit: quote!(log("commit");),
        }
    }
}

#[test]
fn generated_call_enforces_order_across_validation_and_delivery_failures() {
    let items = syn::parse_file("pub fn add(a:i32,b:i32)->i32 { log(\"source\"); a+b }").unwrap();
    let mut flat = Flat::<syn::Type>::parse(
        items
            .items
            .clone()
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap();
    let f = flat.function_mut("add").unwrap();
    f.params[0].ty.slot = Some(Input(0).plan());
    f.params[1].ty.slot = Some(Input(1).plan());
    f.ret.slot = Some(Output.plan());
    let call = flat.function("add").unwrap();
    let body = emit_call(
        call,
        &(),
        &source_binding(),
        &syn::parse_quote!(_),
        &test_parameter_names(call),
    )
    .unwrap();
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

fn record_model() -> Flat {
    let items =
        syn::parse_file("pub struct Point { pub x:f64, pub y:f64 } pub fn point()->Point {}")
            .unwrap();
    let mut flat = Flat::parse(items.items.into_iter().map(|i| (i, Default::default()))).unwrap();
    let result = source_result_expr();
    flat.function_mut("point").unwrap().ret.slot = Some(OutputPlan {
        fallible: false,
        context: Default::default(),
        delivery: Default::default(),
        // Slot code owns field projection; callers only see ordered carriers and values.
        outcomes: prebindgen_flat::Outcomes {
            success: prebindgen_flat::WireNode::Leaf(
                ["y", "x"]
                    .into_iter()
                    .map(|name| Wire {
                        name: format_ident!("wire_{name}"),
                        kind: syn::parse_quote!(f64),
                    })
                    .collect::<Vec<_>>(),
            ),
            source_error: None,
            binding_failure: None,
        },
        statements: TokenStream::new(),
        values: vec![
            (syn::parse_quote!(wire_y), quote!(#result.y)),
            (syn::parse_quote!(wire_x), quote!(#result.x)),
        ],
        commit: TokenStream::new(),
    });
    flat
}

#[test]
fn output_expressions_need_no_field_mapping() {
    let flat = record_model();
    let call = flat.function("point").unwrap();
    let values: Vec<_> = test_return_packet(call, &(), &source_binding())
        .unwrap()
        .map(|out| out.value.to_string())
        .collect();
    assert_eq!(values, ["__result . y", "__result . x"]);
    // The same prepared slot and context work with a caller-selected binding.
    let source = syn::parse_quote!(another_result);
    let values: Vec<_> = test_return_packet(call, &(), &source)
        .unwrap()
        .map(|out| out.value.to_string())
        .collect();
    assert_eq!(values, ["another_result . y", "another_result . x"]);
}

#[test]
fn generation_rejects_inconsistent_output_arity() {
    let mut flat = record_model();
    flat.function_mut("point")
        .unwrap()
        .ret
        .slot
        .as_mut()
        .unwrap()
        .values
        .pop();
    assert!(
        test_return_packet(flat.function("point").unwrap(), &(), &source_binding())
            .unwrap_err()
            .contains("missing output binding")
    );
}

#[test]
fn partial_projection_is_self_contained() {
    let mut flat = record_model();
    let output = flat
        .function_mut("point")
        .unwrap()
        .ret
        .slot
        .as_mut()
        .unwrap();
    let retained = prebindgen_flat::wire_definitions(std::iter::once(&output.outcomes.success))
        .next()
        .unwrap()
        .clone();
    output.outcomes.success = prebindgen_flat::WireNode::Leaf(vec![retained]);
    output.values.pop();
    assert_eq!(
        test_return_packet(flat.function("point").unwrap(), &(), &source_binding())
            .unwrap()
            .len(),
        1
    );
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
    let mut extra = prebindgen_flat::wire_definitions(std::iter::once(&output.outcomes.success))
        .next()
        .unwrap()
        .clone();
    extra.name = syn::parse_quote!(additional_wire);
    let prebindgen_flat::WireNode::Leaf(wires) = &mut output.outcomes.success else {
        panic!()
    };
    wires.push(extra);
    output
        .values
        .push((syn::parse_quote!(additional_wire), quote!(0)));
    assert_eq!(
        test_return_packet(flat.function("point").unwrap(), &(), &source_binding())
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn shared_acknowledgement_is_optional_and_conflicts_are_rejected() {
    let items = syn::parse_file("pub fn add(a:i32,b:i32)->i32 { a+b }").unwrap();
    let mut flat =
        Flat::<syn::Type>::parse(items.items.into_iter().map(|i| (i, Default::default()))).unwrap();
    let f = flat.function_mut("add").unwrap();
    f.params[0].ty.slot = Some(Input(0).plan());
    f.params[1].ty.slot = Some(Input(1).plan());
    f.ret.slot = Some(Output.plan());
    let both = emit_call(
        f,
        &(),
        &source_binding(),
        &syn::parse_quote!(_),
        &test_parameter_names(f),
    )
    .unwrap()
    .to_string();
    f.params[0].ty.slot.as_mut().unwrap().acknowledgement = None;
    assert_eq!(
        emit_call(
            f,
            &(),
            &source_binding(),
            &syn::parse_quote!(_),
            &test_parameter_names(f)
        )
        .unwrap()
        .to_string(),
        both
    );
    f.params[0].ty.slot.as_mut().unwrap().acknowledgement = Some(quote!(different_ack()));
    assert!(emit_call(
        f,
        &(),
        &source_binding(),
        &syn::parse_quote!(_),
        &test_parameter_names(f)
    )
    .unwrap_err()
    .contains("conflicting input acknowledgements"));
    for param in &mut f.params {
        param.ty.slot.as_mut().unwrap().acknowledgement = None;
    }
    assert!(!emit_call(
        f,
        &(),
        &source_binding(),
        &syn::parse_quote!(_),
        &test_parameter_names(f)
    )
    .unwrap()
    .to_string()
    .contains("acknowledge"));
}

#[test]
fn function_accessors_borrow_non_clone_wire_metadata() {
    struct NonClone;
    impl WireKind for NonClone {
        fn rust_type(&self) -> syn::Type {
            syn::parse_quote!(i32)
        }
    }
    let items = syn::parse_file("pub fn echo(x:i32)->i32 { x }").unwrap();
    let mut flat = Flat::<NonClone>::parse(
        items
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )
    .unwrap();
    let wire = || Wire {
        name: syn::parse_quote!(x),
        kind: NonClone,
    };
    let f = flat.function_mut("echo").unwrap();
    f.params[0].ty.slot = Some(InputPlan {
        fallible: false,
        context: Default::default(),
        node: prebindgen_flat::WireNode::Leaf(vec![wire()]),
        acknowledgement: None,
        validation: quote!(),
        acquire: quote!(),
        resources: vec![],
        statements: quote!(),
        argument: quote!(x),
    });
    f.ret.slot = Some(OutputPlan {
        fallible: false,
        context: Default::default(),
        outcomes: prebindgen_flat::Outcomes {
            success: prebindgen_flat::WireNode::Leaf(vec![wire()]),
            source_error: None,
            binding_failure: None,
        },
        statements: quote!(),
        values: vec![(wire().name, source_result_expr())],
        commit: quote!(),
        delivery: Delivery {
            statements: quote!(),
            result: source_result_expr(),
        },
    });
    let call = flat.function("echo").unwrap();
    let f = flat.function("echo").unwrap();
    assert!(std::ptr::eq(
        call.input_wire_definitions().unwrap().next().unwrap(),
        prebindgen_flat::wire_definitions(std::iter::once(
            &f.params[0].ty.slot.as_ref().unwrap().node
        ))
        .next()
        .expect("logical wire index out of range")
    ));
    assert!(std::ptr::eq(
        test_parameters(call).unwrap().next().unwrap(),
        prebindgen_flat::wire_definitions(std::iter::once(
            &f.params[0].ty.slot.as_ref().unwrap().node
        ))
        .next()
        .expect("logical wire index out of range")
    ));
    assert!(std::ptr::eq(
        test_return_packet(call, &(), &source_binding())
            .unwrap()
            .next()
            .unwrap()
            .wire,
        prebindgen_flat::wire_definitions(std::iter::once(
            &f.ret.slot.as_ref().unwrap().outcomes.success
        ))
        .next()
        .expect("logical wire index out of range")
    ));
    let body = emit_call(
        call,
        &(),
        &source_binding(),
        &syn::parse_quote!(_),
        &test_parameter_names(call),
    )
    .unwrap()
    .to_string();
    assert!(body.contains(&quote!(let __prebindgen_argument_0 = x;).to_string()));
    assert!(
        body.contains(&quote!(let __result = crate::echo(__prebindgen_argument_0);).to_string())
    );
    assert!(body.contains(&quote!(Ok(__result)).to_string()));
}

#[test]
fn emit_call_rejects_missing_input_and_result_slots() {
    let items = syn::parse_file("pub fn echo(x:i32)->i32 { x }").unwrap();
    let mut flat =
        Flat::<syn::Type>::parse(items.items.into_iter().map(|i| (i, Default::default()))).unwrap();
    let function = flat.function_mut("echo").unwrap();
    assert!(emit_call(
        function,
        &(),
        &source_binding(),
        &syn::parse_quote!(_),
        &test_parameter_names(function)
    )
    .unwrap_err()
    .contains("parameter `x` has no conversion"));
    function.params[0].ty.slot = Some(Input(0).plan());
    assert!(emit_call(
        function,
        &(),
        &source_binding(),
        &syn::parse_quote!(_),
        &test_parameter_names(function)
    )
    .unwrap_err()
    .contains("result has no conversion"));
}

fn source_binding() -> syn::Ident {
    syn::parse_quote!(__result)
}

// Defined by the consumer, with no Clone/Debug/Default requirement.
struct TestContext {
    runtime: syn::Ident,
    sink: syn::Ident,
    accepted: syn::Ident,
    carrier: syn::Path,
}
impl TemplateContext for TestContext {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        use quote::ToTokens;
        render_template(code, &|name| match name {
            "runtime" => Ok(self.runtime.to_token_stream()),
            "sink" => Ok(self.sink.to_token_stream()),
            "accepted" => Ok(self.accepted.to_token_stream()),
            "carrier" => Ok(self.carrier.to_token_stream()),
            _ => Err(format!("unknown emission parameter `{name}`")),
        })
    }
}

#[test]
fn the_same_slots_execute_with_two_explicit_wrapper_environments() {
    let items = syn::parse_file("pub fn echo(x:i32)->i32 { x + 1 }").unwrap();
    let mut flat = Flat::<syn::Type, TestContext>::parse(
        items
            .items
            .into_iter()
            .map(|item| (item, Default::default())),
    )
    .unwrap();
    let env = template_parameter("runtime");
    let ack = template_parameter("accepted");
    let out = template_parameter("sink");
    let ty = template_parameter("carrier");
    let value = source_result_expr();
    let function = flat.function_mut("echo").unwrap();
    let wire = || Wire {
        name: syn::parse_quote!(incoming),
        kind: syn::parse_quote!(i32),
    };
    function.params[0].ty.slot = Some(InputPlan {
        fallible: false,
        context: Default::default(),
        node: prebindgen_flat::WireNode::Leaf(vec![wire()]),
        acknowledgement: Some(quote!(*#ack = true;)),
        validation: quote!(if #env < 0 { return Err("environment".into()); }),
        acquire: quote!(),
        resources: vec![],
        statements: quote!(let converted = incoming + #env;),
        argument: quote!(converted),
    });
    function.ret.slot = Some(OutputPlan {
        fallible: false,
        context: Default::default(),
        outcomes: prebindgen_flat::Outcomes {
            success: prebindgen_flat::WireNode::Leaf(vec![wire()]),
            source_error: None,
            binding_failure: None,
        },
        statements: quote!(),
        values: vec![(wire().name, value.clone())],
        commit: quote!(),
        delivery: Delivery {
            statements: quote!(unsafe { #out.write(#ty { value: #value }); }),
            result: value,
        },
    });
    let mut wrappers = TokenStream::new();
    for (suffix, type_name) in [("a", "First"), ("b", "Second")] {
        let name = format_ident!("run_{suffix}");
        let environment = format_ident!("runtime_{suffix}");
        let output_pointer = format_ident!("sink_{suffix}");
        let acknowledgement = format_ident!("accepted_{suffix}");
        let output_type = format_ident!("{type_name}");
        let context = TestContext {
            runtime: environment.clone(),
            sink: output_pointer.clone(),
            carrier: syn::parse_quote!(crate::#output_type),
            accepted: acknowledgement.clone(),
        };
        let source = format_ident!("computed_{suffix}");
        let body = emit_call(
            function,
            &context,
            &source,
            &syn::parse_quote!(_),
            &test_parameter_names(function),
        )
        .unwrap();
        assert!(!body.to_string().contains("__prebindgen_emit"));
        wrappers.extend(quote! {
            struct #output_type { value: i32 }
            fn #name(incoming:i32, #environment:i32, #output_pointer:*mut #output_type, #acknowledgement:&mut bool) -> Result<i32,String> { #body }
        });
    }
    let code = quote! {
        fn echo(x:i32)->i32 { x + 1 }
        #wrappers
        fn main() {
            let mut first = First { value: 0 }; let mut second = Second { value: 0 };
            let mut accepted = false;
            assert_eq!(run_a(40, 1, &mut first, &mut accepted), Ok(42));
            assert!(accepted); assert_eq!(first.value, 42);
            accepted = false;
            assert_eq!(run_b(8, 2, &mut second, &mut accepted), Ok(11));
            assert!(accepted); assert_eq!(second.value, 11);
            accepted = false;
            assert_eq!(run_a(40, -1, &mut first, &mut accepted), Err("environment".into()));
            assert!(!accepted); assert_eq!(first.value, 42);
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("contexts.rs");
    let executable = dir.path().join("contexts");
    std::fs::write(&source, code.to_string()).unwrap();
    let compiled = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&source)
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

#[test]
fn emission_markers_do_not_rewrite_identifiers_or_literals_and_require_bindings() {
    let minimal = ();
    assert!(minimal
        .render(&template_parameter("runtime"))
        .unwrap_err()
        .contains("unknown emission parameter `runtime`"));
    let context = TestContext {
        runtime: syn::parse_quote!(runtime),
        sink: syn::parse_quote!(sink),
        accepted: syn::parse_quote!(accepted),
        carrier: syn::parse_quote!(Carrier),
    };
    let environment = template_parameter("runtime");
    let rendered = context
        .render(&quote!({let env = "__prebindgen_emit!(environment)"; (#environment, env)}))
        .unwrap();
    assert_eq!(
        rendered.to_string(),
        quote!({
            let env = "__prebindgen_emit!(environment)";
            (runtime, env)
        })
        .to_string()
    );
}

#[test]
fn alias_checks_accept_opaque_predicates_and_short_circuit_inactive_resources() {
    let code = validate_resources(&[
        Resource {
            registered: false,
            identity: crate::ResourceIdentity::Single(quote!(left)),
            access: Access::Move,
            active: quote!(enabled && count > 0),
        },
        Resource {
            registered: false,
            identity: crate::ResourceIdentity::Single(quote!(
                right.expect("inactive identity must not be evaluated")
            )),
            access: Access::Shared,
            active: quote!(count % 2 == 0),
        },
    ]);
    let mut pair_checks = TokenStream::new();
    let mut assertions = TokenStream::new();
    for (i, first) in [Access::Shared, Access::Exclusive, Access::Move]
        .into_iter()
        .enumerate()
    {
        for (j, second) in [Access::Shared, Access::Exclusive, Access::Move]
            .into_iter()
            .enumerate()
        {
            let name = quote::format_ident!("pair_{i}_{j}");
            let pair = validate_resources(&[
                Resource {
                    registered: false,
                    identity: crate::ResourceIdentity::Single(quote!(left)),
                    access: first,
                    active: quote!(true),
                },
                Resource {
                    registered: false,
                    identity: crate::ResourceIdentity::Single(quote!(right)),
                    access: second,
                    active: quote!(true),
                },
            ]);
            pair_checks.extend(quote! {
                fn #name(left: usize, right: usize) -> Result<(), String> {
                    #pair
                    Ok(())
                }
            });
            let shared = first == Access::Shared && second == Access::Shared;
            assertions.extend(quote! {
                assert_eq!(#name(7, 7).is_ok(), #shared);
                assert!(#name(7, 8).is_ok());
            });
        }
    }
    let source = quote! {
        #pair_checks
        fn check(enabled: bool, count: usize, left: usize, right: Option<usize>) -> Result<(), String> {
            #code
            Ok(())
        }
        fn main() {
            #assertions
            assert!(check(true, 2, 7, Some(7)).is_err());
            assert!(check(true, 2, 7, Some(8)).is_ok());
            assert!(check(false, 2, 7, Some(7)).is_ok());
            assert!(check(true, 1, 7, None).is_ok());
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("predicates.rs");
    let executable = dir.path().join("predicates");
    std::fs::write(&file, source.to_string()).unwrap();
    let compiled = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&file)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(std::process::Command::new(executable)
        .status()
        .unwrap()
        .success());
}

#[test]
fn caller_targets_avoid_surrounding_bindings_and_keep_borrowed_storage_alive() {
    let item = syn::parse_quote!(
        pub fn length(value: &str) -> usize {
            value.len()
        }
    );
    let mut flat = Flat::<syn::Type>::parse([(item, Default::default())]).unwrap();
    let function = flat.function_mut("length").unwrap();
    function.params[0].ty.slot = Some(InputPlan {
        fallible: false,
        context: Default::default(),
        node: prebindgen_flat::WireNode::Leaf(vec![Wire {
            name: syn::parse_quote!(__prebindgen_argument_0),
            kind: syn::parse_quote!(usize),
        }]),
        acknowledgement: None,
        validation: quote!(let __prebindgen_argument_0_ = __prebindgen_argument_0 + 1;),
        acquire: quote!(),
        resources: vec![],
        statements: quote!(let storage = String::from("abc");),
        argument: quote!(storage.as_str()),
    });
    let source = source_result_expr();
    function.ret.slot = Some(OutputPlan {
        fallible: false,
        context: Default::default(),
        outcomes: prebindgen_flat::Outcomes {
            success: Default::default(),
            source_error: None,
            binding_failure: None,
        },
        values: vec![],
        statements: quote!(),
        commit: quote!(),
        delivery: Delivery {
            statements: quote!(),
            result: quote!(#source + __prebindgen_argument_0_),
        },
    });
    let context = ();
    let source = syn::parse_quote!(__prebindgen_argument_0__);
    let body = emit_call(
        function,
        &context,
        &source,
        &syn::parse_quote!(_),
        &test_parameter_names(function),
    )
    .unwrap();
    let target: syn::Ident = syn::parse_quote!(chosen_by_caller);
    let standalone = function.params[0]
        .ty
        .slot
        .as_ref()
        .unwrap()
        .code_fragment_slot(
            InputFragment::Construct {
                context: &context,
                target: &target,
                next: quote!(assert_eq!(#target, "abc")),
            },
            &syn::parse_quote!(()),
        )
        .unwrap();
    let code = quote! {
        fn length(value: &str) -> usize { value.len() }
        fn wrapper(__prebindgen_argument_0: usize) -> Result<usize, String> { #body }
        fn main() {
            assert_eq!(wrapper(10), Ok(14));
            #standalone

        }
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("targets.rs");
    let executable = dir.path().join("targets");
    std::fs::write(&file, code.to_string()).unwrap();
    let compiled = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&file)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(std::process::Command::new(executable)
        .status()
        .unwrap()
        .success());
}

fn validate_resources(resources: &[Resource]) -> TokenStream {
    let plan = InputPlan::<syn::Type> {
        fallible: false,
        context: Default::default(),
        node: Default::default(),
        acknowledgement: None,
        validation: quote!(),
        acquire: quote!(),
        statements: quote!(),
        argument: quote!(()),
        resources: resources.to_vec(),
    };
    let mut aliases = crate::staged::ParameterAliasCheck::default();
    let contribution = plan
        .code_fragment_slot(
            InputFragment::CheckAliases {
                context: &(),
                check: &mut aliases,
            },
            &syn::parse_quote!(()),
        )
        .unwrap();
    let check = aliases.emit().unwrap();
    quote!(let _: () = #contribution; #check;)
}

#[derive(Debug)]
struct TestPacketValue<'a, W: prebindgen_flat::WireKind> {
    wire: &'a Wire<W>,
    value: TokenStream,
}
fn test_return_packet<
    'a,
    W: prebindgen_flat::WireKind,
    I: prebindgen_flat::Conversion,
    C: crate::TemplateContext + 'static,
>(
    f: &'a prebindgen_flat::Function<I, OutputPlan<W, C>>,
    context: &C,
    source: &syn::Ident,
) -> prebindgen_flat::Result<std::vec::IntoIter<TestPacketValue<'a, W>>> {
    let plan = f.output_conversion()?;
    crate::match_output_fields(
        prebindgen_flat::wire_definitions(std::iter::once(&plan.outcomes.success)),
        &plan.values,
    )?
    .into_iter()
    .map(|(wire, value)| {
        Ok(TestPacketValue {
            wire,
            value: crate::emission::render_output(context, value, source)?,
        })
    })
    .collect::<prebindgen_flat::Result<Vec<_>>>()
    .map(Vec::into_iter)
}
fn test_parameters<I: prebindgen_flat::InputConversion, O: prebindgen_flat::Conversion>(
    f: &prebindgen_flat::Function<I, O>,
) -> prebindgen_flat::Result<std::vec::IntoIter<&Wire<I::Wire>>> {
    Ok(f.input_conversions()?
        .into_iter()
        .flat_map(|p| prebindgen_flat::wire_definitions(std::iter::once(p.input_node())))
        .collect::<Vec<_>>()
        .into_iter())
}
fn test_parameter_names<I: prebindgen_flat::InputConversion, O: prebindgen_flat::Conversion>(
    f: &prebindgen_flat::Function<I, O>,
) -> Vec<syn::Ident> {
    test_parameters(f)
        .map(|wires| wires.map(|w| w.name.clone()).collect())
        .unwrap_or_default()
}

#[test]
fn result_fields_match_named_expressions_independently_of_binding_order() {
    let flat = record_model();
    let mut plan = flat
        .function("point")
        .unwrap()
        .output_conversion()
        .unwrap()
        .clone();
    plan.values.reverse();
    let matched = crate::match_output_fields(
        prebindgen_flat::wire_definitions(std::iter::once(&plan.outcomes.success)),
        &plan.values,
    )
    .unwrap();
    assert_eq!(matched[0].0.name, "wire_y");
    assert_eq!(matched[0].1.to_string(), plan.values[1].1.to_string());
    assert_eq!(matched[1].0.name, "wire_x");
    let mut broken = plan.values.clone();
    broken.pop();
    assert!(crate::match_output_fields(
        prebindgen_flat::wire_definitions(std::iter::once(&plan.outcomes.success)),
        &broken
    )
    .unwrap_err()
    .contains("missing"));
    broken = plan.values.clone();
    broken.push((syn::parse_quote!(unknown), quote!(0)));
    assert!(crate::match_output_fields(
        prebindgen_flat::wire_definitions(std::iter::once(&plan.outcomes.success)),
        &broken
    )
    .unwrap_err()
    .contains("unexpected"));
    broken = plan.values.clone();
    broken.push(broken[0].clone());
    assert!(crate::match_output_fields(
        prebindgen_flat::wire_definitions(std::iter::once(&plan.outcomes.success)),
        &broken
    )
    .unwrap_err()
    .contains("duplicate"));
}

#[test]
fn impossible_alias_conflicts_need_no_heap_and_real_conflicts_still_fail() {
    let single = |access| Resource {
        registered: false,
        identity: ResourceIdentity::Single(quote!(left)),
        access,
        active: quote!(enabled),
    };
    let many = |access| Resource {
        registered: false,
        // Dynamic-set scratch storage must not shadow this validation local.
        identity: ResourceIdentity::Many(quote!(__prebindgen_resources.iter().copied())),
        access,
        active: quote!(enabled),
    };
    let mut wrappers = TokenStream::new();
    for (name, resources) in [
        ("no_resources", vec![vec![], vec![]]),
        ("shared", vec![vec![single(Access::Shared)], vec![]]),
        ("exclusive", vec![vec![single(Access::Exclusive)], vec![]]),
        ("moved", vec![vec![single(Access::Move)], vec![]]),
        ("shared_set", vec![vec![many(Access::Shared)], vec![]]),
        ("exclusive_set", vec![vec![many(Access::Exclusive)], vec![]]),
        (
            "internal_pair",
            vec![vec![single(Access::Move), single(Access::Shared)], vec![]],
        ),
        (
            "separate",
            vec![vec![single(Access::Move)], vec![single(Access::Shared)]],
        ),
    ] {
        let mut flat = Flat::<syn::Type>::parse(
            syn::parse_file("pub fn source(a:i32,b:i32)->i32 { 42 }")
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap();
        let function = flat.function_mut("source").unwrap();
        for (index, resources) in resources.into_iter().enumerate() {
            let mut input = Input(index).plan();
            input.acknowledgement = None;
            input.validation = quote!();
            input.acquire = quote!();
            input.statements = quote!();
            input.argument = quote!(1i32);
            input.resources = resources;
            function.params[index].ty.slot = Some(input);
        }
        let mut output = Output.plan();
        output.statements = quote!();
        output.commit = quote!();
        output.delivery.statements = quote!();
        function.ret.slot = Some(output);
        let body = emit_call(
            function,
            &(),
            &source_binding(),
            &syn::parse_quote!(i32),
            &[],
        )
        .unwrap();
        if matches!(
            name,
            "no_resources" | "shared" | "exclusive" | "moved" | "shared_set"
        ) {
            assert!(!body.to_string().contains("Vec"), "{name}: {body}");
        }
        let name = format_ident!("{name}");
        wrappers.extend(quote! {
            fn #name(enabled: bool, left: usize, __prebindgen_resources: &[usize]) -> Result<i32,String> { #body }
        });
    }
    let code = quote! {
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        struct Allocator;
        static COUNTING: AtomicBool = AtomicBool::new(false);
        static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
        #[global_allocator] static ALLOCATOR: Allocator = Allocator;
        unsafe impl GlobalAlloc for Allocator {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                if COUNTING.load(Ordering::Relaxed) { ALLOCATIONS.fetch_add(1, Ordering::Relaxed); }
                System.alloc(layout)
            }
            unsafe fn dealloc(&self, p: *mut u8, layout: Layout) { System.dealloc(p,layout); }
        }
        fn source(_:i32,_:i32)->i32 {42}
        #wrappers
        fn main() {
            COUNTING.store(true, Ordering::Relaxed);
            for _ in 0..1000 {
                let left=std::hint::black_box(7);
                assert_eq!(no_resources(true,left,&[7,7]),Ok(42));
                assert_eq!(shared(true,left,&[7,7]),Ok(42));
                assert_eq!(exclusive(true,left,&[7,7]),Ok(42));
                assert_eq!(moved(true,left,&[7,7]),Ok(42));
                assert_eq!(shared_set(true,left,&[7,7]),Ok(42));
            }
            COUNTING.store(false, Ordering::Relaxed);
            assert_eq!(ALLOCATIONS.load(Ordering::Relaxed),0);
            assert!(exclusive_set(true,7,&[7,7]).is_err());
            assert_eq!(exclusive_set(true,7,&[7,8]),Ok(42));
            assert_eq!(exclusive_set(false,7,&[7,7]),Ok(42));
            assert!(internal_pair(true,7,&[]).is_err());
            assert_eq!(internal_pair(false,7,&[]),Ok(42));
            assert!(separate(true,7,&[]).is_err());
            assert_eq!(separate(false,7,&[]),Ok(42));
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("alias_cost.rs");
    let executable = dir.path().join("alias_cost");
    std::fs::write(&path, code.to_string()).unwrap();
    let compile = std::process::Command::new("rustc")
        .args(["--edition=2021", "-O"])
        .arg(&path)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = std::process::Command::new(executable).output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn fixed_parameter_sets_allocate_nothing_and_reject_only_active_conflicts() {
    use crate::staged::ParameterAliasCheck;
    let mut wrappers = TokenStream::new();
    let mut successes = TokenStream::new();
    let mut conflicts = TokenStream::new();
    for count in [1usize, 2, 3, 8, 9, 16] {
        for shared in [true, false] {
            let mut check = ParameterAliasCheck::default();
            for index in 0..count {
                check
                    .include_parameter(
                        &(),
                        &[Resource {
                            registered: false,
                            identity: ResourceIdentity::Single(
                                quote!(__prebindgen_alias_0[#index]),
                            ),
                            active: quote!(__prebindgen_resources & (1u32 << #index) != 0),
                            access: if shared || (count > 1 && index == 0) {
                                Access::Shared
                            } else if index % 2 == 0 {
                                Access::Move
                            } else {
                                Access::Exclusive
                            },
                        }],
                    )
                    .unwrap();
            }
            let check = check.emit().unwrap();
            assert!(!check.to_string().contains("Vec"), "{check}");
            if shared || count <= 8 {
                assert!(!check.to_string().contains("sort_unstable"), "{check}");
            } else {
                assert!(check.to_string().contains("sort_unstable"), "{check}");
            }
            if shared {
                assert_eq!(check.to_string(), "()");
            }
            let name = format_ident!("fixed_{count}_{shared}");
            wrappers.extend(quote! {
                fn #name(__prebindgen_resources:u32, __prebindgen_alias_0:&[usize]) -> Result<(),String> {
                    let _: () = #check;
                    Ok(())
                }
            });
            let active = (1u32 << count) - 1;
            successes.extend(quote! {
                assert_eq!(#name(#active, std::hint::black_box(&[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15])),Ok(()));
                assert_eq!(#name(0, &[]),Ok(()));
            });
            if count > 1 {
                conflicts.extend(quote! {
                    assert_eq!(#name(#active, &[0;16]).is_ok(),#shared);
                    assert_eq!(#name(1, &[0;16]),Ok(()));
                });
            }
        }
    }
    // A shared/shared duplicate remains legal inside a set that also has a
    // distinct writer. Exercise both direct comparisons and the stack sort.
    for count in [3usize, 9] {
        let mut check = ParameterAliasCheck::default();
        for index in 0..count {
            check
                .include_parameter(
                    &(),
                    &[Resource {
                        registered: false,
                        identity: ResourceIdentity::Single(quote!(__prebindgen_resources[#index])),
                        active: quote!(true),
                        access: if index == 2 {
                            Access::Exclusive
                        } else {
                            Access::Shared
                        },
                    }],
                )
                .unwrap();
        }
        let body = check.emit().unwrap();
        let name = format_ident!("mixed_duplicates_{count}");
        wrappers.extend(quote! {
            fn #name(__prebindgen_resources:&[usize])->Result<(),String> {
                let _: ()=#body;
                Ok(())
            }
        });
        successes.extend(quote! {
            assert_eq!(#name(std::hint::black_box(&[7,7,9,10,11,12,13,14,15])),Ok(()));
        });
        conflicts.extend(quote! {
            assert!(#name(&[7,7,7,10,11,12,13,14,15]).is_err());
        });
    }
    let code = quote! {
        use std::alloc::{GlobalAlloc, Layout, System};
        use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        struct Allocator;
        static COUNTING: AtomicBool=AtomicBool::new(false);
        static ALLOCATIONS: AtomicUsize=AtomicUsize::new(0);
        #[global_allocator] static ALLOCATOR:Allocator=Allocator;
        unsafe impl GlobalAlloc for Allocator {
            unsafe fn alloc(&self,layout:Layout)->*mut u8 {
                if COUNTING.load(Ordering::Relaxed) {ALLOCATIONS.fetch_add(1,Ordering::Relaxed);}
                System.alloc(layout)
            }
            unsafe fn dealloc(&self,p:*mut u8,layout:Layout) {System.dealloc(p,layout);}
        }
        #wrappers
        fn main() {
            COUNTING.store(true,Ordering::Relaxed);
            for _ in 0..1000 { #successes }
            COUNTING.store(false,Ordering::Relaxed);
            assert_eq!(ALLOCATIONS.load(Ordering::Relaxed),0);
            #conflicts
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("fixed_aliases.rs");
    let executable = dir.path().join("fixed_aliases");
    std::fs::write(&source, code.to_string()).unwrap();
    let compile = std::process::Command::new("rustc")
        .args(["--edition=2021", "-O"])
        .arg(&source)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let run = std::process::Command::new(executable).output().unwrap();
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
}

#[test]
fn eager_resource_conversions_reject_deferred_resource_neighbors() {
    let items = syn::parse_file("pub fn add(a:i32,b:i32)->i32 {a+b}").unwrap();
    let mut flat =
        Flat::<syn::Type>::parse(items.items.into_iter().map(|i| (i, Default::default()))).unwrap();
    let function = flat.function_mut("add").unwrap();
    for (index, parameter) in function.params.iter_mut().enumerate() {
        let mut plan = Input(index).plan();
        plan.resources.push(Resource {
            registered: index == 0,
            identity: ResourceIdentity::Single(quote!(8usize)),
            access: Access::Move,
            active: quote!(true),
        });
        parameter.ty.slot = Some(plan);
    }
    function.ret.slot = Some(Output.plan());
    let function = flat.function("add").unwrap();
    let error = emit_call(
        function,
        &(),
        &source_binding(),
        &syn::parse_quote!(_),
        &test_parameter_names(function),
    )
    .unwrap_err();
    assert!(
        error.contains("cannot mix deferred resource conversions"),
        "{error}"
    );
}
