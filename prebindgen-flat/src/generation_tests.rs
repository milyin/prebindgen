//! Generation-contract tests use only flat, without the preparation crate.
use super::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

struct Input(usize);
impl Input {
    fn plan(&self) -> InputPlan<&'static str> {
        let index = self.0;
        let name = format_ident!("arg{index}");
        let validate = format!("validate{index}");
        let acquire = format!("acquire{index}");
        let construct = format!("construct{index}");
        InputPlan {
            context: Default::default(),
            acknowledgement: Some(
                quote!(log("acknowledge"); if fail_acknowledge { return Err("acknowledgement".into()); }),
            ),
            wires: vec![],
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
    fn plan(&self) -> OutputPlan<&'static str> {
        OutputPlan {
            context: Default::default(),
            delivery: Delivery {
                statements: quote!(log("deliver"); if fail_delivery { return Err("delivery".into()); }),
                result: source_result_expr(),
            },
            wires: vec![Wire {
                name: syn::parse_quote!(out),
                kind: "int",
            }],
            statements: quote!(log("prepare");),
            values: vec![source_result_expr()],
            commit: quote!(log("commit");),
        }
    }
}

#[test]
fn generated_call_enforces_order_across_validation_and_delivery_failures() {
    let items = syn::parse_file("pub fn add(a:i32,b:i32)->i32 { log(\"source\"); a+b }").unwrap();
    let mut flat = Flat::<InputPlan<&str>, OutputPlan<&str>>::parse(
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
    let body = emit_call(call, &emission_context()).unwrap();
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

fn record_model() -> Flat<InputPlan<()>, OutputPlan<&'static str>> {
    let items =
        syn::parse_file("pub struct Point { pub x:f64, pub y:f64 } pub fn point()->Point {}")
            .unwrap();
    let mut flat = Flat::parse(items.items.into_iter().map(|i| (i, Default::default()))).unwrap();
    let result = source_result_expr();
    flat.function_mut("point").unwrap().ret.slot = Some(OutputPlan {
        context: Default::default(),
        delivery: Default::default(),
        // Slot code owns field projection; callers only see ordered carriers and values.
        wires: ["y", "x"]
            .into_iter()
            .map(|name| Wire {
                name: format_ident!("wire_{name}"),
                kind: "Double",
            })
            .collect(),
        statements: TokenStream::new(),
        values: vec![quote!(#result.y), quote!(#result.x)],
        commit: TokenStream::new(),
    });
    flat
}

#[test]
fn output_expressions_need_no_field_mapping() {
    let flat = record_model();
    let call = flat.function("point").unwrap();
    let values: Vec<_> = call
        .outputs(&emission_context())
        .unwrap()
        .map(|out| out.value.to_string())
        .collect();
    assert_eq!(values, ["__result . y", "__result . x"]);
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
        emit_call(flat.function("point").unwrap(), &emission_context())
            .err()
            .unwrap()
            .contains("count differs")
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
    output.wires.pop();
    output.values.pop();
    assert_eq!(
        flat.function("point")
            .unwrap()
            .outputs(&emission_context())
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
    let mut extra = output.wires[0].clone();
    extra.name = syn::parse_quote!(additional_wire);
    output.wires.push(extra);
    output.values.push(quote!(0));
    assert_eq!(
        flat.function("point")
            .unwrap()
            .outputs(&emission_context())
            .unwrap()
            .len(),
        3
    );
}

#[test]
fn shared_acknowledgement_is_optional_and_conflicts_are_rejected() {
    let items = syn::parse_file("pub fn add(a:i32,b:i32)->i32 { a+b }").unwrap();
    let mut flat = Flat::<InputPlan<&str>, OutputPlan<&str>>::parse(
        items.items.into_iter().map(|i| (i, Default::default())),
    )
    .unwrap();
    let f = flat.function_mut("add").unwrap();
    f.params[0].ty.slot = Some(Input(0).plan());
    f.params[1].ty.slot = Some(Input(1).plan());
    f.ret.slot = Some(Output.plan());
    let both = emit_call(f, &emission_context()).unwrap().to_string();
    f.params[0].ty.slot.as_mut().unwrap().acknowledgement = None;
    assert_eq!(emit_call(f, &emission_context()).unwrap().to_string(), both);
    f.params[0].ty.slot.as_mut().unwrap().acknowledgement = Some(quote!(different_ack()));
    assert!(emit_call(f, &emission_context())
        .unwrap_err()
        .contains("conflicting input acknowledgements"));
    for param in &mut f.params {
        param.ty.slot.as_mut().unwrap().acknowledgement = None;
    }
    assert!(!emit_call(f, &emission_context())
        .unwrap()
        .to_string()
        .contains("acknowledge"));
}

#[test]
fn function_accessors_borrow_non_clone_wire_metadata() {
    struct NonClone;
    let items = syn::parse_file("pub fn echo(x:i32)->i32 { x }").unwrap();
    let mut flat = Flat::<InputPlan<NonClone>, OutputPlan<NonClone>>::parse(
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
        context: Default::default(),
        wires: vec![wire()],
        acknowledgement: None,
        validation: quote!(),
        acquire: quote!(),
        resources: vec![],
        statements: quote!(),
        argument: quote!(x),
    });
    f.ret.slot = Some(OutputPlan {
        context: Default::default(),
        wires: vec![wire()],
        statements: quote!(),
        values: vec![source_result_expr()],
        commit: quote!(),
        delivery: Delivery {
            statements: quote!(),
            result: source_result_expr(),
        },
    });
    let call = flat.function("echo").unwrap();
    let f = flat.function("echo").unwrap();
    assert!(std::ptr::eq(
        call.inputs().unwrap().next().unwrap(),
        &f.params[0].ty.slot.as_ref().unwrap().wires[0]
    ));
    assert!(std::ptr::eq(
        call.outputs(&emission_context())
            .unwrap()
            .next()
            .unwrap()
            .wire,
        &f.ret.slot.as_ref().unwrap().wires[0]
    ));
    assert_eq!(
        emit_call(call, &emission_context()).unwrap().to_string(),
        quote!(let __result = crate::echo(x); Ok(__result)).to_string()
    );
}

#[test]
fn emit_call_rejects_missing_input_and_result_slots() {
    let items = syn::parse_file("pub fn echo(x:i32)->i32 { x }").unwrap();
    let mut flat = Flat::<InputPlan<&str>, OutputPlan<&str>>::parse(
        items.items.into_iter().map(|i| (i, Default::default())),
    )
    .unwrap();
    let function = flat.function_mut("echo").unwrap();
    assert!(emit_call(function, &emission_context())
        .unwrap_err()
        .contains("parameter `x` has no conversion"));
    function.params[0].ty.slot = Some(Input(0).plan());
    assert!(emit_call(function, &emission_context())
        .unwrap_err()
        .contains("result has no conversion"));
}

fn emission_context() -> syn::Ident {
    syn::parse_quote!(__result)
}

// Defined by the consumer, with no Clone/Debug/Default requirement.
struct TestContext {
    result: syn::Ident,
    runtime: syn::Ident,
    sink: syn::Ident,
    accepted: syn::Ident,
    carrier: syn::Path,
}
impl EmissionContext for TestContext {
    fn source_result(&self) -> &syn::Ident {
        &self.result
    }
}
impl TemplateContext for TestContext {
    fn render(&self, code: &TokenStream) -> Result<TokenStream> {
        use quote::ToTokens;
        render_template(code, &|name| match name {
            "source_result" => Ok(self.result.to_token_stream()),
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
    let mut flat = Flat::<InputPlan<(), TestContext>, OutputPlan<(), TestContext>>::parse(
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
        kind: (),
    };
    function.params[0].ty.slot = Some(InputPlan {
        context: Default::default(),
        wires: vec![wire()],
        acknowledgement: Some(quote!(*#ack = true;)),
        validation: quote!(if #env < 0 { return Err("environment".into()); }),
        acquire: quote!(),
        resources: vec![],
        statements: quote!(let converted = incoming + #env;),
        argument: quote!(converted),
    });
    function.ret.slot = Some(OutputPlan {
        context: Default::default(),
        wires: vec![wire()],
        statements: quote!(),
        values: vec![value.clone()],
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
            result: format_ident!("computed_{suffix}"),
            runtime: environment.clone(),
            sink: output_pointer.clone(),
            carrier: syn::parse_quote!(crate::#output_type),
            accepted: acknowledgement.clone(),
        };
        let body = emit_call(function, &context).unwrap();
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
    let minimal: syn::Ident = syn::parse_quote!(computed);
    assert!(minimal
        .render(&template_parameter("runtime"))
        .unwrap_err()
        .contains("unknown emission parameter `runtime`"));
    let context = TestContext {
        result: minimal,
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
fn custom_conversions_use_context_without_template_rendering() {
    struct Context {
        local: syn::Ident,
        increment: i32,
    }
    impl EmissionContext for Context {
        fn source_result(&self) -> &syn::Ident {
            &self.local
        }
    }
    struct Input;
    impl InputConversion for Input {
        type Wire = ();
        type Context = Context;
        fn wires(&self) -> &[Wire<()>] {
            &[]
        }
        fn resources(&self, _: &Context) -> Result<Vec<Resource>> {
            Ok(vec![])
        }
        fn validation(&self, _: &Context) -> Result<TokenStream> {
            Ok(quote!())
        }
        fn acquisition(&self, _: &Context) -> Result<TokenStream> {
            Ok(quote!())
        }
        fn construction(&self, _: &Context) -> Result<TokenStream> {
            Ok(quote!())
        }
        fn argument(&self, context: &Context) -> Result<TokenStream> {
            let increment = context.increment;
            Ok(quote!(#increment))
        }
    }
    struct Output;
    impl OutputConversion for Output {
        type Wire = ();
        type Context = Context;
        fn wires(&self) -> &[Wire<()>] {
            &[]
        }
        fn values(&self, _: &Context) -> Result<Vec<TokenStream>> {
            Ok(vec![])
        }
        fn preparation(&self, _: &Context) -> Result<TokenStream> {
            Ok(quote!())
        }
        fn delivery(&self, _: &Context) -> Result<TokenStream> {
            Ok(quote!())
        }
        fn commit(&self, _: &Context) -> Result<TokenStream> {
            Ok(quote!())
        }
        fn result(&self, context: &Context) -> Result<TokenStream> {
            let local = &context.local;
            Ok(quote!(#local))
        }
    }
    let item = syn::parse_quote!(
        pub fn echo(x: i32) -> i32 {
            x
        }
    );
    let mut flat = Flat::<Input, Output>::parse([(item, Default::default())]).unwrap();
    let function = flat.function_mut("echo").unwrap();
    function.params[0].ty.slot = Some(Input);
    function.ret.slot = Some(Output);
    let context = Context {
        local: syn::parse_quote!(returned),
        increment: 42,
    };
    assert_eq!(
        emit_call(function, &context).unwrap().to_string(),
        quote!(let returned = crate::echo(42i32); Ok(returned)).to_string()
    );
}

#[test]
fn alias_checks_accept_opaque_predicates_and_short_circuit_inactive_resources() {
    let code = validate_resources(&[
        Resource {
            identity: quote!(left),
            access: Access::Move,
            active: quote!(enabled && count > 0),
        },
        Resource {
            identity: quote!(right.expect("inactive identity must not be evaluated")),
            access: Access::Shared,
            active: quote!(count % 2 == 0),
        },
    ]);
    let source = quote! {
        fn check(enabled: bool, count: usize, left: usize, right: Option<usize>) -> Result<(), String> {
            #code
            Ok(())
        }
        fn main() {
            assert!(check(true, 2, 7, Some(7)).is_err());
            assert!(check(true, 2, 7, Some(8)).is_ok());
            assert!(check(false, 2, 7, None).is_ok());
            assert!(check(true, 1, 7, None).is_ok());
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("predicates.rs");
    let executable = dir.path().join("predicates");
    std::fs::write(&file, source.to_string()).unwrap();
    let compiled = std::process::Command::new("rustc")
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
