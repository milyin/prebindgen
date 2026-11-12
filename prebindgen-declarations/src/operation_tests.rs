//! Contract tests for the operation API, including a four-method implementation.
use super::*;
use prebindgen_conversions::{staged::emit_call, value};

struct Scalars;
impl Codec for Scalars {
    type Schema = prebindgen_conversions::PlanSchema<Type>;
    type Wire = Type;
    type Context = ();
    type Extra = ();
    type Chain = ();
    type Branch = ();

    fn input_leaf(&self, r: InputLeaf<'_, Self>) -> Result<CodecInput<Self>> {
        if !same_type(r.ty, &syn::parse_quote!(i64)) {
            return Err("this codec only supports i64".into());
        }
        let wire = &r.wire;
        value::input(r.ty, r.wire.clone(), r.target, quote!(#wire), r.ty.clone())
    }
    fn output_leaf(&self, r: OutputLeaf<'_, Self>) -> Result<CodecOutput<Self>> {
        if !same_type(r.ty, &syn::parse_quote!(i64)) {
            return Err("this codec only supports i64".into());
        }
        value::output(
            r.ty,
            r.wire,
            r.value,
            r.access.is_shared(),
            |v| v,
            r.ty.clone(),
        )
    }
    fn output_consumes(&self, _: &Flat<Self::Schema>, _: &Type, _: &()) -> bool {
        false
    }
    fn group(&self, kind: GroupKind<'_>) -> Result<()> {
        match kind {
            GroupKind::Values | GroupKind::Fields | GroupKind::Field(_) => Ok(()),
            GroupKind::NamedOutput(_) => Err("named output views are unsupported".into()),
        }
    }
}
fn model() -> Flat<<Scalars as Codec>::Schema> {
    Flat::parse(
        syn::parse_file("pub fn add(a:i64,b:i64)->i64{}")
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap()
}
#[test]
fn four_method_codec_compiles_and_executes_a_complete_call() {
    let mut flat = model();
    Declarations::new(&flat, vec![], vec![], vec![])
        .unwrap()
        .fill(&mut flat, &Scalars, &fun!(add), &())
        .unwrap();
    // The wrapper selects its final result independently of codec preparation.
    let output = flat.function_mut("add").unwrap().ret.slot.as_mut().unwrap();
    output.delivery.result = output.values[0].1.clone();
    let f = flat.function("add").unwrap();
    let params = f
        .params
        .iter()
        .flat_map(|p| {
            prebindgen_flat::wire_definitions(std::iter::once(&p.ty.slot.as_ref().unwrap().node))
        })
        .map(|w| w.name.clone())
        .collect::<Vec<_>>();
    let parameter_types = params.iter().map(|p| quote!(#p:i64));
    let body = emit_call(
        f,
        &(),
        &syn::parse_quote!(result),
        &syn::parse_quote!(i64),
        &params,
    )
    .unwrap();
    let code = quote! {
        fn add(a:i64,b:i64)->i64 {a+b}
        fn bridge(#(#parameter_types),*)->Result<i64,String> {#body}
        fn main() {assert_eq!(bridge(20,22).unwrap(),42);}
    };
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("scalar.rs");
    let binary = dir.path().join("scalar");
    std::fs::write(&source, code.to_string()).unwrap();
    let build = std::process::Command::new("rustc")
        .arg("--edition=2021")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()
        .unwrap();
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    assert!(std::process::Command::new(binary)
        .status()
        .unwrap()
        .success());
}
#[test]
fn minimal_codec_rejects_optional_features_and_bulk_before_decoding() {
    let flat = model();
    let ty: Type = syn::parse_quote!(i64);
    // Scalars deliberately ignores representation in its leaves. The compiler
    // must enforce opt-in before passing a bulk leaf or packing a sequence.
    for (source, input) in [
        (
            ty.clone(),
            Input::Unary {
                operation: InputUnary::Bulk,
                child: Box::new(Input::Leaf(InputLeafOp::Value)),
            },
        ),
        (
            syn::parse_quote!(()),
            Input::Unary {
                operation: InputUnary::Bulk,
                child: Box::new(Input::Branch {
                    operation: InputBranch::Fields,
                    children: vec![],
                }),
            },
        ),
        (
            syn::parse_quote!(Vec<i64>),
            Input::Unary {
                operation: InputUnary::Bulk,
                child: Box::new(Input::Unary {
                    operation: InputUnary::Sequence,
                    child: Box::new(Input::Leaf(InputLeafOp::Value)),
                }),
            },
        ),
    ] {
        let error = compile_callback_return(&flat, &Scalars, &source, &input, &()).unwrap_err();
        assert!(error.contains("bulk inputs are unsupported"), "{error}");
    }
    for (source, output) in [
        (
            ty.clone(),
            Output::Unary {
                operation: OutputUnary::Bulk,
                child: Box::new(Output::Leaf(OutputLeafOp::Value)),
            },
        ),
        (
            ty.clone(),
            Output::Unary {
                operation: OutputUnary::Bulk,
                child: Box::new(Output::Leaf(OutputLeafOp::Projected(Projection::owned()))),
            },
        ),
        (
            syn::parse_quote!(()),
            Output::Unary {
                operation: OutputUnary::Bulk,
                child: Box::new(Output::Branch {
                    operation: OutputBranch::Fields,
                    children: vec![],
                }),
            },
        ),
        (
            syn::parse_quote!(Vec<i64>),
            Output::Unary {
                operation: OutputUnary::Bulk,
                child: Box::new(Output::Unary {
                    operation: OutputUnary::Sequence {
                        delivery: SequenceDelivery::Value,
                        mode: None,
                    },
                    child: Box::new(Output::Leaf(OutputLeafOp::Value)),
                }),
            },
        ),
    ] {
        let error = compile_output_value(&flat, &Scalars, &source, &output, &()).unwrap_err();
        assert!(error.contains("bulk outputs are unsupported"), "{error}");
    }
    let error = compile_callback_return(
        &flat,
        &Scalars,
        &syn::parse_quote!(Option<i64>),
        &Input::Unary {
            operation: InputUnary::Optional,
            child: Box::new(Input::Leaf(InputLeafOp::Value)),
        },
        &(),
    )
    .unwrap_err();
    assert!(error.contains("optional conversion is unsupported"));
    assert!(Scalars.selector(&()).unwrap_err().contains("unsupported"));
    assert!(Scalars.group(GroupKind::NamedOutput("view")).is_err());
    assert!(Scalars
        .sequence_input(SequenceInputRequest {
            view: prebindgen_flat::SequenceView::Owned,
            extra: &(),
            wire: format_ident!("rows"),
            length: format_ident!("len"),
            index: format_ident!("index"),
            element: &WireNode::Leaf(vec![]),
            active: &Condition::default(),
            representation: Representation::Ordinary
        })
        .is_err());
}
#[test]
fn default_projection_keeps_its_reference_and_field_checks() {
    let flat = model();
    let scalar = syn::parse_quote!(i64);
    let request = || {
        OutputLeaf::new(
            &flat,
            &scalar,
            &(),
            format_ident!("wire"),
            quote!(value),
            ValueAccess::Owned,
        )
    };
    let error = Scalars
        .output_projected(
            request(),
            &Projection::owned().field("missing", Projection::owned()),
        )
        .unwrap_err();
    assert!(error.contains("field projection"));
    let ty = syn::parse_quote!(Option<&i64>);
    let error = Scalars
        .output_projected(
            OutputLeaf::new(
                &flat,
                &ty,
                &(),
                format_ident!("wire"),
                quote!(value),
                ValueAccess::Owned,
            ),
            &Projection::owned(),
        )
        .unwrap_err();
    assert!(error.contains("cannot consume a reference payload"));
}

// The probe opts into bulk so the test can observe the compiler's exact dispatch
// contexts without depending on either language's wire representation.
#[derive(Default)]
struct Probe(
    std::cell::RefCell<Vec<(Representation, InputRole)>>,
    std::cell::Cell<bool>,
);
impl Codec for Probe {
    type Schema = <Scalars as Codec>::Schema;
    type Wire = Type;
    type Context = ();
    type Extra = ();
    type Chain = ();
    type Branch = ();
    fn input_leaf(&self, r: InputLeaf<'_, Self>) -> Result<CodecInput<Self>> {
        self.0.borrow_mut().push((r.representation, r.role));
        Scalars.input_leaf(InputLeaf {
            flat: r.flat,
            ty: r.ty,
            extra: r.extra,
            wire: r.wire,
            target: r.target,
            active: r.active,
            representation: r.representation,
            role: r.role,
        })
    }
    fn output_leaf(&self, r: OutputLeaf<'_, Self>) -> Result<CodecOutput<Self>> {
        Scalars.output_leaf(OutputLeaf {
            flat: r.flat,
            ty: r.ty,
            extra: r.extra,
            wire: r.wire,
            value: r.value,
            access: r.access,
            representation: r.representation,
        })
    }
    fn output_consumes(&self, _: &Flat<Self::Schema>, _: &Type, _: &()) -> bool {
        false
    }
    fn group(&self, k: GroupKind<'_>) -> Result<()> {
        Scalars.group(k)
    }
    fn supports_bulk(&self, direction: Direction) -> bool {
        direction == Direction::Input || self.1.get()
    }
}
#[test]
fn input_context_preserves_all_dispatch_cases_and_callback_roles() {
    let flat = model();
    let codec = Probe::default();
    let ty = syn::parse_quote!(i64);
    for role in [InputRole::Parameter, InputRole::CallbackResult] {
        for (bulk, representation) in [
            (false, Representation::Ordinary),
            (true, Representation::Bulk),
        ] {
            let compiler = Compiler {
                input_role: role.into(),
                ..Compiler::new(&flat, &codec, "arg".into())
            };
            compiler.bulk.set(bulk);
            compiler
                .input(
                    &ty,
                    &Input::Leaf(InputLeafOp::Value),
                    &(),
                    &Condition::default(),
                )
                .unwrap();
            assert_eq!(codec.0.borrow().last(), Some(&(representation, role)));
        }
    }
    codec.0.borrow_mut().clear();
    compile_callback_return(
        &flat,
        &codec,
        &syn::parse_quote!(()),
        &Input::Branch {
            operation: InputBranch::Fields,
            children: vec![],
        },
        &(),
    )
    .unwrap();
    compile_callback_return(
        &flat,
        &codec,
        &ty,
        &Input::constructor(
            "add",
            [
                Input::Leaf(InputLeafOp::Value),
                Input::Leaf(InputLeafOp::Value),
            ],
        ),
        &(),
    )
    .unwrap();
    assert_eq!(codec.0.borrow().len(), 2);
    assert!(codec
        .0
        .borrow()
        .iter()
        .all(|r| r.1 == InputRole::CallbackResult));
    let error = compile_output_value(
        &flat,
        &codec,
        &ty,
        &Output::Unary {
            operation: OutputUnary::Bulk,
            child: Box::new(Output::Leaf(OutputLeafOp::Value)),
        },
        &(),
    )
    .unwrap_err();
    assert!(error.contains("bulk outputs are unsupported"));
}

struct MalformedOptional(u8);
impl Codec for MalformedOptional {
    type Schema = <Scalars as Codec>::Schema;
    type Wire = Type;
    type Context = ();
    type Extra = ();
    type Chain = ();
    type Branch = ();
    fn input_leaf(&self, r: InputLeaf<'_, Self>) -> Result<CodecInput<Self>> {
        Scalars.input_leaf(InputLeaf::new(
            r.flat, r.ty, r.extra, r.wire, r.target, r.active,
        ))
    }
    fn output_leaf(&self, r: OutputLeaf<'_, Self>) -> Result<CodecOutput<Self>> {
        Scalars.output_leaf(OutputLeaf::new(
            r.flat, r.ty, r.extra, r.wire, r.value, r.access,
        ))
    }
    fn output_consumes(&self, _: &Flat<Self::Schema>, _: &Type, _: &()) -> bool {
        false
    }
    fn group(&self, k: GroupKind<'_>) -> Result<()> {
        Scalars.group(k)
    }
    fn optional(&self, r: Optional<'_, Self>) -> Result<OptionalEncoding<Self>> {
        let presence = match self.0 {
            0 => Presence::Flag(Wire {
                name: format_ident!("wrong"),
                kind: syn::parse_quote!(i32),
            }),
            1 => Presence::Niche { input_test: None },
            2 => Presence::Niche {
                input_test: Some(quote!(true)),
            },
            _ => Presence::Flag(Wire {
                name: r.presence,
                kind: syn::parse_quote!(i32),
            }),
        };
        let absent_values = match self.0 {
            3 => vec![(format_ident!("unexpected"), quote!(0i64))],
            4 => vec![],
            _ => prebindgen_flat::wire_definitions(std::iter::once(&*r.payload))
                .map(|w| (w.name.clone(), quote!(0i64)))
                .collect(),
        };
        Ok(OptionalEncoding {
            presence,
            metadata: (),
            absent_values: if r.direction == Direction::Input && self.0 != 3 {
                vec![]
            } else {
                absent_values
            },
        })
    }
}
#[test]
fn malformed_optional_encodings_are_rejected_before_installing_a_slot() {
    let mut flat = Flat::<<Scalars as Codec>::Schema>::parse(
        syn::parse_file("pub fn f(value:Option<i64>)->Option<i64>{}")
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap();
    for (mode, input, diagnostic) in [
        (0, true, "requested presence binding"),
        (1, true, "presence test only for input"),
        (3, true, "cannot contain absent output"),
        (0, false, "requested presence binding"),
        (2, false, "presence test only for input"),
        (3, false, "missing absent output binding"),
        (4, false, "missing absent output binding"),
    ] {
        let result = if input {
            bind_input(
                &mut flat,
                &MalformedOptional(mode),
                "f",
                "value",
                &Input::Unary {
                    operation: InputUnary::Optional,
                    child: Box::new(Input::Leaf(InputLeafOp::Value)),
                },
                &(),
            )
        } else {
            bind_output(
                &mut flat,
                &MalformedOptional(mode),
                "f",
                &Output::Unary {
                    operation: OutputUnary::Optional,
                    child: Box::new(Output::Leaf(OutputLeafOp::Value)),
                },
                &(),
            )
        };
        let error = result.unwrap_err();
        assert!(error.contains(diagnostic), "{error}");
        assert!(flat.function("f").unwrap().params[0].ty.slot.is_none());
        assert!(flat.function("f").unwrap().ret.slot.is_none());
    }
}

#[test]
fn enabled_bulk_still_rejects_enclosing_callback_delivery_before_element_lowering() {
    let flat = model();
    let codec = Probe::default();
    codec.1.set(true);
    let recipe = Output::Unary {
        operation: OutputUnary::Bulk,
        child: Box::new(Output::Unary {
            operation: OutputUnary::Sequence {
                delivery: SequenceDelivery::Callback,
                mode: None,
            },
            child: Box::new(Output::Leaf(OutputLeafOp::Value)),
        }),
    };
    let error = compile_output_value(&flat, &codec, &syn::parse_quote!(Vec<i64>), &recipe, &())
        .unwrap_err();
    assert!(
        error.contains("bulk cannot enclose callback delivery"),
        "{error}"
    );
}

#[test]
fn resolved_flat_compiles_after_its_configuration_is_dropped() {
    let flat = model();
    let declarations = Declarations::new(&flat, vec![], vec![], vec![]).unwrap();
    let configuration = vec![fun!(add)];
    let resolved = declarations
        .resolve(flat, Scalars, &configuration, ())
        .unwrap();
    drop(configuration);
    drop(declarations);
    assert!(matches!(
        resolved.function("add").unwrap().params[0].ty.slot,
        Some(ResolvedSlot::Tree(Input::Leaf(_)))
    ));
    let compiled = compile_resolved(resolved).unwrap();
    prebindgen_flat::validate_bindings(&compiled).unwrap();
    assert_eq!(compiled.function("add").unwrap().params.len(), 2);
}

#[test]
fn none_is_unit_and_cannot_silently_discard_non_unit_values() {
    let source = model();
    let defaults = Declarations::new(&source, vec![], vec![], vec![]).unwrap();
    assert!(defaults
        .resolve_input(
            &source,
            &Scalars,
            &syn::parse_quote!(i64),
            &Input::None,
            &()
        )
        .is_err());
    assert!(defaults
        .resolve_output(
            &source,
            &Scalars,
            &syn::parse_quote!(i64),
            &Output::None,
            &()
        )
        .is_err());
    for tree in [Input::auto(), Input::fields([]), Input::None] {
        assert!(matches!(
            defaults
                .resolve_input(&source, &Scalars, &syn::parse_quote!(()), &tree, &())
                .unwrap(),
            Input::None
        ));
    }
    for tree in [Output::auto(), Output::fields([]), Output::None] {
        assert!(matches!(
            defaults
                .resolve_output(&source, &Scalars, &syn::parse_quote!(()), &tree, &())
                .unwrap(),
            Output::None
        ));
    }
}
