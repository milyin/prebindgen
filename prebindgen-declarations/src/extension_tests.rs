//! An independent adapter extends the shared declaration family without C/JNI operations.
use crate::*;
use prebindgen_flat::{type_key, Flat};
use std::{collections::BTreeMap, convert::Infallible};

#[derive(Clone, Debug)]
struct Incoming;
#[derive(Clone, Debug)]
struct Outgoing;
impl InputOps for Incoming {
    type Leaf = String;
    type Unary = String;
    type Branch = String;
    type Arrow = Box<CallbackOperation<Settings>>;
}
impl OutputOps for Outgoing {
    type Leaf = String;
    type Unary = String;
    type Branch = String;
    type Arrow = Infallible;
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Settings(&'static str);
type Input = InputNode<Incoming, Outgoing>;
type Output = OutputNode<Incoming, Outgoing>;
type Function = DeclFunction<Input, Output, Settings>;
type Parameter = DeclExpandParam<Input, Function, Settings>;
type Return = DeclExpandReturn<Output, Function, Settings>;
type Fields = DeclFields<Output, Settings>;
type Error = DeclError<Function, Settings>;
type Expansion = DeclExpand<Parameter, Return, Error>;

macro_rules! adapter_fun {
    ($name:ident) => {
        Function::new(syn::parse_quote!($name))
    };
}

fn parameter() -> Parameter {
    Parameter::new(
        syn::parse_quote!(Value),
        |function| Input::Leaf(function.source_name()),
        || Input::Leaf("direct".into()),
        |ty, alternatives, extension| {
            let child = alternatives.build(
                |children| Input::Branch {
                    operation: "choose".into(),
                    children,
                },
                || Input::Leaf("default".into()),
                |child| Input::Unary {
                    operation: "temporary".into(),
                    child: Box::new(child),
                },
            );
            Input::Unary {
                operation: format!("{}:{}", type_key(&ty), extension.unwrap().0),
                child: Box::new(child),
            }
        },
    )
}
fn returned() -> Return {
    Return::new(
        syn::parse_quote!(Value),
        |function| {
            getter_view(function, Output::Leaf, |name, child| Output::Unary {
                operation: name,
                child: Box::new(child),
            })
        },
        || Output::Leaf("direct".into()),
        |policy| Output::Leaf(format!("{:?}", policy.mode)),
        |child| Output::Unary {
            operation: "owned fields".into(),
            child: Box::new(child),
        },
        |ty, children, extension| Output::Branch {
            operation: format!("{}:{}", type_key(&ty), extension.unwrap().0),
            children,
        },
    )
}
fn fields() -> Fields {
    Fields::new(
        syn::parse_quote!(accessor),
        |output| match output {
            Output::Branch { children, .. } if children.is_empty() => {
                Output::Leaf("discard".into())
            }
            other => other,
        },
        |function, overrides, extension| Output::Branch {
            operation: format!("{function}:{}", extension.unwrap().0),
            children: overrides.into_values().collect(),
        },
    )
}
trait ParameterSettings {
    fn packet_format(self) -> Self;
}
impl ParameterSettings for Parameter {
    fn packet_format(self) -> Self {
        self.with_extension(Settings("packet"))
    }
}
fn model() -> Flat {
    Flat::parse(syn::parse_file("pub type Value=String;pub fn call(value:i64)->i64{}pub fn rows()->Vec<i64>{}pub fn register(cb:impl Fn(i64)->i64){}").unwrap().items.into_iter().map(|i|(i,Default::default()))).unwrap()
}

#[test]
fn adapter_methods_compose_with_common_input_and_return_builders() {
    let input: Input = parameter()
        .packet_format()
        .variant(adapter_fun!(make))
        .variant_self()
        .reconstruct()
        .into();
    let Input::Unary { operation, child } = input else {
        panic!("lost typed scope")
    };
    assert_eq!(operation, "Value:packet");
    let Input::Unary { operation, child } = *child else {
        panic!("lost reconstruction")
    };
    assert_eq!(operation, "temporary");
    let Input::Branch { children, .. } = *child else {
        panic!("lost alternatives")
    };
    assert!(matches!(&children[..],[Input::Leaf(a),Input::Leaf(b)] if a=="make" && b=="direct"));
    let empty: Input = parameter().reconstruct().packet_format().into();
    let Input::Unary { child, .. } = empty else {
        panic!("lost scope")
    };
    let Input::Unary { child, .. } = *child else {
        panic!("lost temporary")
    };
    assert!(matches!(*child, Input::Leaf(ref value) if value == "default"));

    let output: Output = returned()
        .with_extension(Settings("result"))
        .field(Function::new(syn::parse_quote!(getter)).name("renamed"))
        .field_self_ref()
        .field_self_into()
        .into();
    let Output::Branch {
        operation,
        children,
    } = output
    else {
        panic!("lost views")
    };
    assert_eq!(operation, "Value:result");
    assert!(
        matches!(&children[..],[Output::Unary {operation,..},Output::Leaf(borrowed),Output::Leaf(owned)] if operation=="renamed" && borrowed=="Borrowed" && owned=="Owned")
    );
}
#[test]
fn field_options_and_omission_survive_common_builders_and_registration() {
    let empty = Output::Branch {
        operation: "empty".into(),
        children: vec![],
    };
    let output: Output = returned()
        .with_extension(Settings("result"))
        .fields_into(
            fields()
                .with_extension(Settings("fields"))
                .field("omitted", empty),
        )
        .into();
    let Output::Branch { children, .. } = output else {
        panic!("lost views")
    };
    let Output::Unary { operation, child } = &children[0] else {
        panic!("lost consuming accessor")
    };
    assert_eq!(operation, "owned fields");
    assert!(
        matches!(child.as_ref(),Output::Branch {operation,children} if operation=="accessor:fields" && matches!(&children[..],[Output::Leaf(value)] if value=="discard"))
    );
    let declarations: Vec<Expansion> = vec![
        parameter().packet_format().variant_self().into(),
        returned()
            .with_extension(Settings("result"))
            .field_self()
            .into(),
        Error::new(
            syn::parse_quote!(Failure),
            Function::new(syn::parse_quote!(message)),
        )
        .with_extension(Settings("error"))
        .into(),
    ];
    assert!(
        matches!(&declarations[0],Expansion::Param(p) if p.extension==Some(Settings("packet")))
    );
    assert!(
        matches!(&declarations[1],Expansion::Return(r) if r.extension==Some(Settings("result")))
    );
    assert!(matches!(&declarations[2],Expansion::Error(e) if e.extension==Some(Settings("error"))));
}
#[test]
fn function_settings_survive_common_methods_resolution_and_callback_variants() {
    let mut flat = model();
    let declaration: Function = crate::fun!(call)
        .with_extension(Settings("function"))
        .name("Call")
        .expand_param("value", Input::Leaf("in".into()))
        .expand_return(Output::Leaf("out".into()));
    let recipe = resolve_function(
        &flat,
        &declaration,
        |_, tree| Ok(tree.unwrap().clone()),
        |_| Ok(declaration.extension.as_ref().unwrap().0),
    )
    .unwrap();
    assert!(matches!(&recipe.inputs[..],[Some(Input::Leaf(s))] if s=="in"));
    assert_eq!(recipe.output, Some("function"));
    let declaration = Function::new(syn::parse_quote!(rows))
        .with_extension(Settings("function"))
        .callback_return_only();
    let variant = callback_variant(&mut flat, &declaration, "rows_each", |_, _| {
        Output::Leaf("each".into())
    })
    .unwrap();
    assert_eq!(variant.extension, Some(Settings("function")));
    assert!(variant.callback_return.is_none());
}
#[test]
fn callback_settings_inherit_override_and_reach_the_resolved_arrow() {
    type Callback = DeclCallback<Input, Output, Settings>;
    let flat = model();
    let ty = syn::parse_quote!(impl Fn(i64) -> i64);
    let default = Callback::new(ty)
        .with_extension(Settings("default"))
        .name("Callback")
        .expand_param(0, Output::Leaf("argument".into()))
        .expand_return(Input::Leaf("result".into()));
    for extension in [None, Some(Settings("local"))] {
        let mut local = Callback::new(default.ty.clone());
        local.extension = extension.clone();
        let (operation, arguments, result) = resolve_callback(
            &flat,
            &default.ty,
            &local,
            Some(&default),
            &BTreeMap::new(),
            &Input::None,
            &Output::None,
            &mut vec![],
            |_, input, _| Ok(input.clone()),
            |_, output, _| Ok(output.clone()),
        )
        .unwrap();
        assert_eq!(operation.extension, extension.or(Some(Settings("default"))));
        let (arguments, result) = resolve_callback_arrow(
            &flat,
            &default.ty,
            &operation,
            &arguments,
            &result,
            |_, v| Ok(v.clone()),
            |_, v| Ok(v.clone()),
        )
        .unwrap();
        let arrow = Input::Arrow {
            operation: Box::new(operation),
            domain: Box::new(Output::Branch {
                operation: "arguments".into(),
                children: arguments,
            }),
            codomain: Box::new(result),
        };
        assert!(matches!(arrow,Input::Arrow {operation,..} if operation.extension.is_some()));
    }
    let operation = CallbackOperation {
        ty: default.ty.clone(),
        foreign: None,
        namespace: None,
        explicit_return: false,
        extension: Some(Settings("prepared")),
    };
    let error = resolve_callback_arrow(
        &flat,
        &default.ty,
        &operation,
        &[],
        &Input::None,
        |_, _: &Output| -> prebindgen_flat::Result<Output> {
            panic!("invalid arity must be rejected before conversion")
        },
        |_, _| Ok(Input::None),
    )
    .unwrap_err();
    assert!(error.contains("arity"));
}
#[test]
fn canonical_conversion_preparation_preserves_nonclone_adapter_settings() {
    struct ConversionSettings(String);
    let mut flat = model();
    let declaration = convert!(Value)
        .input(DeclConvertSource::input(syn::parse_quote!(String), false))
        .output(DeclConvertSource::output(syn::parse_quote!(String), false))
        .with_extension(ConversionSettings("conversion".into()));
    prepare_value_conversions(&mut flat, std::slice::from_ref(&declaration)).unwrap();
    assert_eq!(declaration.extension.as_ref().unwrap().0, "conversion");
    assert!(flat
        .type_info(&flat.resolve_type(&declaration.ty).unwrap())
        .unwrap()
        .conversion
        .input
        .is_some());
}
#[test]
#[should_panic(expected = "conversion helpers only support")]
fn decorated_helpers_cannot_silently_discard_adapter_options() {
    parameter()
        .variant(Function::new(syn::parse_quote!(make)).with_extension(Settings("binding only")));
}

#[test]
fn independent_transport_extension_uses_the_shared_fluent_api() {
    let input: TestInput<TestInputRequest, TestOutputRequest> =
        test_expand_param!(i64).variant_self().bulk().into();
    let output: TestOutput<TestInputRequest, TestOutputRequest> =
        test_expand_return!(i64).field_self().bulk().into();
    assert!(
        matches!(input,TestInput::Request(TestInputRequest::ForType {input,..}) if matches!(*input,TestInput::Unary {operation:TestInputUnary::Bulk,..}))
    );
    assert!(
        matches!(output,TestOutput::Request(TestOutputRequest::ForType {output,..}) if matches!(*output,TestOutput::Unary {operation:TestOutputUnary::Bulk,..}))
    );
}
#[test]
fn empty_and_unnamed_views_keep_their_original_meaning() {
    let input: Input = parameter().packet_format().into();
    assert!(
        matches!(input,Input::Unary {child,..} if matches!(child.as_ref(),Input::Branch {children,..} if children.is_empty()))
    );
    let output: Output = returned()
        .with_extension(Settings("return"))
        .field(Function::new(syn::parse_quote!(unnamed)))
        .into();
    assert!(
        matches!(output,Output::Branch {children,..} if matches!(&children[..],[Output::Leaf(name)] if name=="unnamed"))
    );
}
#[test]
fn cloning_builders_does_not_require_clonable_function_declarations() {
    struct MoveOnly(String);
    let input = DeclExpandParam::<String, MoveOnly>::new(
        syn::parse_quote!(Value),
        |f| f.0,
        || "direct".into(),
        |_, alternatives, _| {
            alternatives.build(
                |values| values.join(","),
                || "default".into(),
                |value| value,
            )
        },
    );
    assert_eq!(
        input.clone().variant(MoveOnly("input".into())).into_input(),
        "input"
    );
    assert!(format!("{input:?}").contains("DeclExpandParam"));
    let output = DeclExpandReturn::<String, MoveOnly>::new(
        syn::parse_quote!(Value),
        |f| f.0,
        || "direct".into(),
        |_| "projection".into(),
        |v| v,
        |_, views, _| views.join(","),
    );
    assert_eq!(
        output
            .clone()
            .field(MoveOnly("output".into()))
            .into_output(),
        "output"
    );
    assert!(format!("{output:?}").contains("DeclExpandReturn"));
}

struct ArrowCodec;
impl Codec for ArrowCodec {
    type InOps = Incoming;
    type OutOps = Outgoing;
    type CompileState = ();
    type Schema = prebindgen_conversions::PlanSchema<syn::Type>;
    type Wire = syn::Type;
    type Context = ();
    type Extra = ();
    type Chain = ();
    type Branch = ();
    fn compile_input_tree(
        &self,
        cx: &Compiler<'_, Self>,
        _ty: &syn::Type,
        tree: &ResolvedInput<Self>,
        extra: &(),
        active: &Condition,
        names: Option<(&syn::Ident, &syn::Ident)>,
    ) -> prebindgen_flat::Result<CodecInputPlan<Self>> {
        let Input::Arrow {
            operation,
            domain,
            codomain,
        } = tree
        else {
            return Err("expected callback".into());
        };
        self.input_arrow(InputArrow {
            flat: cx.flat,
            operation,
            domain,
            codomain,
            extra,
            wire: names.map_or_else(|| cx.name(), |(wire, _)| wire.clone()),
            target: names.map_or_else(|| cx.name(), |(_, target)| target.clone()),
            active,
        })
    }
    fn compile_output_tree(
        &self,
        cx: &Compiler<'_, Self>,
        ty: &syn::Type,
        _tree: &ResolvedOutput<Self>,
        extra: &(),
        value: proc_macro2::TokenStream,
        borrowed: bool,
        wire: Option<&syn::Ident>,
    ) -> prebindgen_flat::Result<CodecOutputPlan<Self>> {
        self.output_leaf(OutputLeaf::new(
            cx.flat,
            ty,
            extra,
            wire.cloned().unwrap_or_else(|| cx.name()),
            value,
            if borrowed {
                ValueAccess::Shared
            } else {
                ValueAccess::Owned
            },
        ))
    }
    fn input_arrow(
        &self,
        request: InputArrow<'_, Self>,
    ) -> prebindgen_flat::Result<CodecInputPlan<Self>> {
        let offset: i64 = request
            .operation
            .extension
            .as_ref()
            .unwrap()
            .0
            .parse()
            .unwrap();
        let wire = &request.wire;
        let target = &request.target;
        Ok(prebindgen_conversions::InputPlan::new(
            prebindgen_flat::WireNode::Leaf(vec![prebindgen_flat::Wire {
                name: wire.clone(),
                kind: syn::parse_quote!(i64),
            }]),
            quote::quote!(let #target={let base=#wire;move |value:i64|value+base+#offset};),
            quote::quote!(#target),
        ))
    }
    fn input_leaf(
        &self,
        _request: InputLeaf<'_, Self>,
    ) -> prebindgen_flat::Result<CodecInputPlan<Self>> {
        Err("expected callback".into())
    }
    fn output_leaf(
        &self,
        request: OutputLeaf<'_, Self>,
    ) -> prebindgen_flat::Result<CodecOutputPlan<Self>> {
        prebindgen_conversions::value::output(
            request.ty,
            request.wire,
            request.value,
            request.access.is_shared(),
            |v| v,
            request.ty.clone(),
        )
    }
    fn output_consumes(&self, _: &Flat<Self::Schema>, _: &syn::Type, _: &()) -> bool {
        false
    }
    fn group(&self, _: GroupKind<'_>) -> prebindgen_flat::Result<()> {
        Ok(())
    }
    fn finish_output(
        &self,
        plan: &mut CodecOutputPlan<Self>,
        _: &(),
    ) -> prebindgen_flat::Result<()> {
        plan.delivery.result = plan.values[0].1.clone();
        Ok(())
    }
}
#[test]
fn callback_extensions_reach_the_compiler_and_change_generated_behavior() {
    let flat: Flat<<ArrowCodec as Codec>::Schema> = Flat::parse(
        syn::parse_file("pub fn apply(callback:impl Fn(i64)->i64)->i64{}")
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap();
    let callback = DeclCallback::<Input, Output>::new(syn::parse_quote!(impl Fn(i64) -> i64))
        .with_extension(Settings("2"));
    let (operation, arguments, result) = resolve_callback(
        &flat,
        &callback.ty,
        &callback,
        None,
        &BTreeMap::new(),
        &Input::Leaf("result".into()),
        &Output::Leaf("argument".into()),
        &mut vec![],
        |_, v, _| Ok(v.clone()),
        |_, v, _| Ok(v.clone()),
    )
    .unwrap();
    let arrow = Input::Arrow {
        operation: Box::new(operation),
        domain: Box::new(Output::Branch {
            operation: "arguments".into(),
            children: arguments,
        }),
        codomain: Box::new(result),
    };
    let resolved = resolve_bindings(
        flat,
        ArrowCodec,
        &[Function::new(syn::parse_quote!(apply)).with_extension(Settings("function"))],
        |_, _, declaration, _| {
            assert_eq!(declaration.extension, Some(Settings("function")));
            Ok(FunctionRecipe {
                inputs: vec![Some(arrow.clone())],
                output: Some(Output::Leaf("result".into())),
            })
        },
        (),
    )
    .unwrap();
    drop(callback);
    let compiled = compile_resolved(resolved).unwrap();
    let function = compiled.function("apply").unwrap();
    let params = prebindgen_flat::wire_definitions(std::iter::once(
        &function.params[0].ty.slot.as_ref().unwrap().node,
    ))
    .map(|wire| wire.name.clone())
    .collect::<Vec<_>>();
    let definitions = params.iter().map(|name| quote::quote!(#name:i64));
    let body = prebindgen_conversions::staged::emit_call(
        function,
        &(),
        &syn::parse_quote!(result),
        &syn::parse_quote!(i64),
        &params,
    )
    .unwrap();
    let source = quote::quote!(fn apply(callback:impl Fn(i64)->i64)->i64{callback(40)} fn bridge(#(#definitions),*)->Result<i64,String>{#body} fn main(){assert_eq!(bridge(0).unwrap(),42);});
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("callback.rs");
    let executable = dir.path().join("callback");
    std::fs::write(&path, source.to_string()).unwrap();
    let result = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&path)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(std::process::Command::new(executable)
        .status()
        .unwrap()
        .success());
}
