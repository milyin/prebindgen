use super::*;
use prebindgen_conversions::value::{self, Value};
use prebindgen_flat::validate_bindings;
struct TestCodec;
const TEST_CODEC: TestCodec = TestCodec;
#[derive(Clone, Debug, PartialEq)]
enum Kind {
    Value(Value),
    Token,
    Selector,
}
#[derive(Clone, Debug)]
struct TestWire {
    kind: Kind,
    name: &'static str,
    ty: String,
    component: ComponentPath,
}
// Construction-only data needs no Clone or Debug implementation.
#[derive(Default)]
struct Extra {
    name: &'static str,
    copy: bool,
}
fn wire(kind: Kind, ty: &Type, extra: &Extra) -> TestWire {
    TestWire {
        kind,
        name: extra.name,
        ty: type_key(ty),
        component: Default::default(),
    }
}
impl Codec for TestCodec {
    type Extra = Extra;
    type Wire = TestWire;
    type Context = syn::Ident;
    type Input = InputPlan<TestWire>;
    type Output = OutputPlan<TestWire>;
    fn prefix_component(&self, wire: &mut TestWire, segment: ComponentSegment) {
        wire.component.prepend(segment);
    }
    fn selector(&self, extra: &Extra) -> TestWire {
        TestWire {
            kind: Kind::Selector,
            name: extra.name,
            ty: "i32".into(),
            component: Default::default(),
        }
    }
    fn consumes(&self, ty: &Type, extra: &Extra) -> bool {
        Value::of(ty).is_none() && !extra.copy
    }
    fn input_leaf(
        &self,
        ty: &Type,
        extra: &Extra,
        w: syn::Ident,
        v: syn::Ident,
        active: &Condition,
    ) -> Result<InputPlan<TestWire>> {
        let name = extra.name;
        if let Some(value) = Value::of(ty) {
            let decode = quote!(decode(#name, #w)?);
            return value::input(
                ty,
                w,
                v,
                &active.rust(),
                decode,
                wire(Kind::Value(value), ty, extra),
            );
        }
        Ok(InputPlan {
            context: Default::default(),
            acknowledgement: None,
            wires: vec![Wire {
                name: w.clone(),
                kind: wire(Kind::Token, ty, extra),
            }],
            validation: quote!(),
            acquire: quote!(),
            resources: vec![],
            statements: quote!(let #v=receive_token(#w);),
            argument: quote!(#v),
        })
    }
    fn output_leaf(
        &self,
        ty: &Type,
        extra: &Extra,
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<OutputPlan<TestWire>> {
        let name = extra.name;
        if let Some(value) = Value::of(ty) {
            let kind = wire(Kind::Value(value), ty, extra);
            return value::output(ty, w, v, borrowed, |v| quote!(encode(#name, #v)), kind);
        }
        if borrowed && !extra.copy {
            return Err("test token cannot be copied".into());
        }
        Ok(OutputPlan {
            context: Default::default(),
            delivery: Default::default(),
            wires: vec![Wire {
                name: w.clone(),
                kind: wire(Kind::Token, ty, extra),
            }],
            statements: quote!(let #w=send_token(#v);),
            values: vec![quote!(#w)],
            commit: quote!(),
        })
    }
}
fn parse(source: &str) -> Bindings<TestWire> {
    Flat::parse(
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap()
}
fn model() -> Bindings<TestWire> {
    parse("pub struct Payload { pub data: Vec<u8> } pub fn make(s: String) -> Payload {} pub fn foo(x: Payload, y: Payload) -> Payload {}")
}
fn extra(name: &'static str) -> Extra {
    Extra { name, copy: false }
}
fn fill(flat: &mut Bindings<TestWire>, decl: &FunctionDecl) -> Result<()> {
    Declarations::new(flat, vec![])?.fill(flat, &TEST_CODEC, decl, &Extra::default())
}
#[test]
fn different_occurrences_have_different_converters() {
    let mut flat = model();
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &Input::selector([Input::Leaf, Input::constructor("make", [Input::Leaf])]),
        &extra("x"),
    )
    .unwrap();
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "y",
        &Input::Leaf,
        &extra("y"),
    )
    .unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &Output::Leaf,
        &extra("result"),
    )
    .unwrap();
    validate_bindings(&flat).unwrap();
    let f = flat.function("foo").unwrap();
    let x = f.params[0].ty.slot.as_ref().unwrap();
    let y = f.params[1].ty.slot.as_ref().unwrap();
    assert_eq!(x.wires.len(), 3);
    assert_eq!(y.wires.len(), 1);
    assert!(x.wires.iter().all(|w| w.kind.name == "x"));
    assert_eq!(y.wires[0].kind.name, "y");
}
#[test]
fn rejects_incomplete_slots() {
    let mut flat = model();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &Output::Leaf,
        &Extra::default(),
    )
    .unwrap();
    assert!(validate_bindings(&flat).unwrap_err().contains("no input"));
}
#[test]
fn rejects_inputs_without_a_result_converter() {
    let mut flat = model();
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &Input::Leaf,
        &Extra::default(),
    )
    .unwrap();
    assert!(validate_bindings(&flat)
        .unwrap_err()
        .contains("no output converter"));
}
#[test]
fn binding_keeps_helpers_records_locations_and_guards() {
    let mut flat = model();
    let before = flat.clone();
    fill(&mut flat, &fun!(foo)).unwrap();
    validate_bindings(&flat).unwrap();
    let helper = flat.function("make").unwrap();
    assert!(helper.ret.slot.is_none());
    assert!(helper.params.iter().all(|p| p.ty.slot.is_none()));
    let record = before.records.values().next().unwrap();
    let retained = flat.record(&record.ty).unwrap();
    assert_eq!(retained.ty, record.ty);
    assert_eq!(retained.fields[0].name, record.fields[0].name);
    assert_eq!(retained.fields[0].ty, record.fields[0].ty);
    assert_eq!(flat.guards, before.guards);
    assert_eq!(flat.types, before.types);
    assert_eq!(
        format!("{:?}", flat.function("foo").unwrap().location),
        format!("{:?}", before.function("foo").unwrap().location)
    );
}
#[test]
fn failed_rebinding_preserves_existing_converter() {
    let mut flat = model();
    fill(&mut flat, &fun!(foo)).unwrap();
    let before = format!("{flat:?}");
    let error = bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &Input::constructor("foo", []),
        &Extra::default(),
    )
    .unwrap_err();
    assert!(error.contains("argument count"));
    assert_eq!(format!("{flat:?}"), before);
    for output in [
        Output::combine([Output::Leaf, Output::Leaf]),
        Output::fields([Output::Leaf, Output::Leaf]),
    ] {
        assert!(bind_output(&mut flat, &TEST_CODEC, "foo", &output, &Extra::default()).is_err());
        assert_eq!(format!("{flat:?}"), before);
    }
    assert!(bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "missing",
        &Input::Leaf,
        &Extra::default()
    )
    .unwrap_err()
    .contains("unknown parameter"));
}
#[test]
fn observations_precede_move_but_wires_keep_order() {
    let mut flat = model();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &Output::combine([Output::Leaf, Output::fields([Output::Leaf])]),
        &Extra::default(),
    )
    .unwrap();
    let o = flat.function("foo").unwrap().ret.slot.as_ref().unwrap();
    assert!(o.wires[0].kind.kind == Kind::Token);
    let body = o.statements.to_string();
    assert!(body.find("clone").unwrap() < body.find("send_token").unwrap());
}
#[test]
fn declarations_reject_unknown_or_mismatched_field_overrides() {
    let source = "pub struct Payload { pub data: Vec<u8> } pub fn parts(p: &Payload) -> Payload {} pub fn read(p: Payload) -> Payload {}";
    for (field, output, message) in [
        (
            "missing",
            expand_return!(Vec<u8>).field_self(),
            "unknown output field",
        ),
        (
            "data",
            expand_return!(String).field_self(),
            "does not match occurrence",
        ),
    ] {
        let mut flat = parse(source);
        let function = fun!(read)
            .expand_return(expand_return!(Payload).fields(fields!(parts).field(field, output)));
        assert!(fill(&mut flat, &function).unwrap_err().contains(message));
        let f = flat.function("read").unwrap();
        assert!(f.params[0].ty.slot.is_none());
        assert!(f.ret.slot.is_none());
    }
}
#[test]
fn declarations_preserve_converters_and_replace_explicit_overrides() {
    let mut flat = model();
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &Input::selector([Input::constructor("make", [Input::Leaf]), Input::Leaf]),
        &extra("x"),
    )
    .unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &Output::Leaf,
        &extra("result"),
    )
    .unwrap();
    let declarations = Declarations::new(
        &flat,
        vec![expand_param!(Payload)
            .variant(fun!(make))
            .variant_self()
            .into()],
    )
    .unwrap();
    declarations
        .fill(&mut flat, &TEST_CODEC, &fun!(foo), &extra("defaults"))
        .unwrap();
    assert_eq!(
        flat.function("foo").unwrap().params[1]
            .ty
            .slot
            .as_ref()
            .unwrap()
            .wires
            .len(),
        3
    );
    declarations
        .fill(
            &mut flat,
            &TEST_CODEC,
            &fun!(foo).expand_param("y", expand_param!(Payload).variant_self()),
            &extra("override"),
        )
        .unwrap();
    let f = flat.function("foo").unwrap();
    let x = f.params[0].ty.slot.as_ref().unwrap();
    let y = f.params[1].ty.slot.as_ref().unwrap();
    assert_eq!(x.wires.len(), 3);
    assert!(x.wires[2].kind.kind == Kind::Token);
    assert!(x.wires.iter().all(|w| w.kind.name == "x"));
    assert_eq!(y.wires.len(), 1);
    assert_eq!(y.wires[0].kind.name, "override");
    assert_eq!(f.ret.slot.as_ref().unwrap().wires[0].kind.name, "result");
    assert!(x.validation.to_string().contains("\"x\""));
}
#[test]
fn declaration_compilation_is_atomic_even_when_output_fails() {
    let mut flat = model();
    fill(&mut flat, &fun!(foo)).unwrap();
    let before = format!("{flat:?}");
    let function = fun!(foo)
        .expand_param("x", expand_param!(Payload).variant(fun!(make)))
        .expand_return(expand_return!(Payload).field(fun!(make)));
    assert!(fill(&mut flat, &function)
        .unwrap_err()
        .contains("must take exactly"));
    assert_eq!(format!("{flat:?}"), before);
}
#[test]
fn independent_binding_and_rebinding_keeps_names_unique_and_stable() {
    let mut flat = model();
    let mut reversed = model();
    for (model, order) in [(&mut flat, ["x", "y"]), (&mut reversed, ["y", "x"])] {
        for parameter in order {
            bind_input(
                model,
                &TEST_CODEC,
                "foo",
                parameter,
                &Input::Leaf,
                &Extra::default(),
            )
            .unwrap();
        }
        bind_output(model, &TEST_CODEC, "foo", &Output::Leaf, &Extra::default()).unwrap();
    }
    assert_eq!(format!("{flat:?}"), format!("{reversed:?}"));
    let old_y = format!("{:?}", flat.function("foo").unwrap().params[1].ty.slot);
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &Input::selector([Input::Leaf, Input::constructor("make", [Input::Leaf])]),
        &Extra::default(),
    )
    .unwrap();
    let f = flat.function("foo").unwrap();
    assert_eq!(format!("{:?}", f.params[1].ty.slot), old_y);
    let wires = f
        .params
        .iter()
        .flat_map(|p| &p.ty.slot.as_ref().unwrap().wires)
        .chain(&f.ret.slot.as_ref().unwrap().wires)
        .collect::<Vec<_>>();
    let names = wires
        .iter()
        .map(|w| w.name.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(names.len(), wires.len());
}
#[test]
fn field_defaults_follow_type_expansions() {
    let mut flat = parse("pub type Key=String; pub struct Payload { pub key: Key, pub data: Vec<u8> } pub fn parts(p: &Payload) -> Payload {} pub fn key_text(k: &Key) -> String {} pub fn read(p: Payload) -> Payload {}");
    Declarations::new(
        &flat,
        vec![expand_return!(Key).field(fun!(key_text)).into()],
    )
    .unwrap()
    .fill(
        &mut flat,
        &TEST_CODEC,
        &fun!(read).expand_return(expand_return!(Payload).fields(fields!(parts))),
        &Extra::default(),
    )
    .unwrap();
    let output = flat.function("read").unwrap().ret.slot.as_ref().unwrap();
    assert!(output.wires[0].kind.kind == Kind::Value(Value::Text));
    assert!(output.wires[1].kind.kind == Kind::Value(Value::Bytes));
    assert_eq!(output.wires[0].kind.ty, "String");
    assert_eq!(output.wires[1].kind.ty, "Vec < u8 >");
}
#[test]
fn explicit_scalar_identity_is_a_value_conversion() {
    let mut flat = parse("pub fn f(x:i64)->i64{}");
    let defaults = Declarations::new(
        &flat,
        vec![
            expand_param!(i64).variant_self().into(),
            expand_return!(i64).field_self().into(),
        ],
    )
    .unwrap();
    defaults
        .fill(&mut flat, &TEST_CODEC, &fun!(f), &extra("scalar"))
        .unwrap();
    let f = flat.function("f").unwrap();
    let input = f.params[0].ty.slot.as_ref().unwrap();
    let output = f.ret.slot.as_ref().unwrap();
    let scalar = Kind::Value(Value::Scalar(prebindgen_flat::Primitive::I64));
    assert_eq!(input.wires[0].kind.kind, scalar);
    assert_eq!(output.wires[0].kind.kind, scalar);
    assert_eq!(input.wires[0].kind.name, "scalar");
    assert_eq!(input.wires[0].kind.ty, "i64");
    assert_eq!(output.wires[0].kind.name, "scalar");
    assert_eq!(output.wires[0].kind.ty, "i64");
    assert!(input.validation.to_string().contains("scalar"));
    assert!(output.statements.to_string().contains("scalar"));
}
#[test]
fn nested_recipes_use_only_the_callers_extra() {
    let mut flat = parse("pub type Key=String; pub struct Payload { pub key:Key } pub fn make(k:Key)->Payload {} pub fn parts(p:&Payload)->Payload {} pub fn foo(x:Payload,y:Payload)->Payload {}");
    // Independently bound helpers must not affect the enclosing conversion.
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "make",
        "k",
        &Input::Leaf,
        &extra("constructor declaration"),
    )
    .unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "parts",
        &Output::Leaf,
        &extra("getter declaration"),
    )
    .unwrap();
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &Input::selector([Input::Leaf, Input::constructor("make", [Input::Leaf])]),
        &extra("x"),
    )
    .unwrap();
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "y",
        &Input::fields([Input::Leaf]),
        &extra("y"),
    )
    .unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &Output::combine([
            Output::Leaf,
            Output::getter("parts", Output::fields([Output::Leaf])),
        ]),
        &Extra {
            name: "result",
            copy: true,
        },
    )
    .unwrap();
    let f = flat.function("foo").unwrap();
    let foreign = |wires: &[Wire<TestWire>]| {
        wires
            .iter()
            .filter_map(|w| match &w.kind.kind {
                Kind::Token => Some((w.kind.name, w.kind.ty.clone())),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        foreign(&f.params[0].ty.slot.as_ref().unwrap().wires),
        vec![
            ("x", "crate :: Payload".into()),
            ("x", "crate :: Key".into())
        ]
    );
    assert_eq!(
        foreign(&f.params[1].ty.slot.as_ref().unwrap().wires),
        vec![("y", "crate :: Key".into())]
    );
    assert_eq!(
        foreign(&f.ret.slot.as_ref().unwrap().wires),
        vec![
            ("result", "crate :: Payload".into()),
            ("result", "crate :: Key".into())
        ]
    );
}
#[test]
fn extra_controls_consumption_without_changing_the_recipe() {
    let recipe = Output::combine([Output::Leaf, Output::Leaf]);
    for copy in [false, true] {
        let mut flat = model();
        let result = bind_output(
            &mut flat,
            &TEST_CODEC,
            "foo",
            &recipe,
            &Extra {
                copy,
                ..Extra::default()
            },
        );
        if copy {
            assert!(result.is_ok());
        } else {
            assert!(result.unwrap_err().contains("consume the same value twice"));
        }
    }
}

#[test]
fn record_projection_retains_field_mappings_in_prepared_flat() {
    let mut flat = parse("pub struct Point { pub x:i64, pub y:i64 } pub fn point()->Point {}");
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "point",
        &Output::fields([Output::Leaf, Output::Leaf]),
        &Extra::default(),
    )
    .unwrap();
    let call = flat.function("point").unwrap();
    assert_eq!(
        call.outputs(&emission_context())
            .unwrap()
            .map(|out| out.wire.kind.component.clone())
            .collect::<Vec<_>>(),
        ["x", "y"].map(|name| ComponentPath(vec![ComponentSegment::Field(name.into())]))
    );
}

#[test]
fn finalization_runs_once_on_complete_slots_and_preserves_atomicity() {
    struct FinalizingCodec;
    impl Codec for FinalizingCodec {
        type Extra = Extra;
        type Wire = TestWire;
        type Context = syn::Ident;
        type Input = InputPlan<TestWire>;
        type Output = OutputPlan<TestWire>;
        fn prefix_component(&self, wire: &mut TestWire, segment: ComponentSegment) {
            wire.component.prepend(segment);
        }
        fn selector(&self, extra: &Extra) -> TestWire {
            TEST_CODEC.selector(extra)
        }
        fn consumes(&self, ty: &Type, extra: &Extra) -> bool {
            TEST_CODEC.consumes(ty, extra)
        }
        fn input_leaf(
            &self,
            ty: &Type,
            extra: &Extra,
            w: syn::Ident,
            v: syn::Ident,
            active: &Condition,
        ) -> Result<InputPlan<TestWire>> {
            let mut plan = TEST_CODEC.input_leaf(ty, extra, w.clone(), v, active)?;
            plan.acknowledgement = Some(if extra.name == "conflict" {
                quote!(ack(#w);)
            } else {
                quote!(ack();)
            });
            Ok(plan)
        }
        fn output_leaf(
            &self,
            ty: &Type,
            extra: &Extra,
            w: syn::Ident,
            v: TokenStream,
            borrowed: bool,
        ) -> Result<OutputPlan<TestWire>> {
            TEST_CODEC.output_leaf(ty, extra, w, v, borrowed)
        }
        fn finish_input(&self, plan: &mut InputPlan<TestWire>, _: &Extra) -> Result<()> {
            assert_eq!(
                plan.acknowledgement.as_ref().unwrap().to_string(),
                quote!(ack();).to_string()
            );
            assert_eq!(plan.wires.len(), 2);
            Ok(())
        }
        fn finish_output(&self, plan: &mut OutputPlan<TestWire>, extra: &Extra) -> Result<()> {
            if extra.name == "fail" {
                return Err("finalization failed".into());
            }
            assert!(plan.delivery.statements.is_empty());
            assert!(plan.wires.is_empty());
            plan.delivery.statements = quote!(deliver_unit(););
            Ok(())
        }
    }
    let source = "pub struct Pair { pub a:i32, pub b:i32 } pub fn pair(a:i32,b:i32)->Pair {} pub fn take(p:Pair) {}";
    let mut conflicting = parse(source);
    assert!(bind_input(
        &mut conflicting,
        &FinalizingCodec,
        "take",
        "p",
        &Input::constructor("pair", [Input::Leaf, Input::Leaf]),
        &Extra {
            name: "conflict",
            ..Extra::default()
        }
    )
    .unwrap_err()
    .contains("conflicting input acknowledgements"));
    assert!(conflicting.function("take").unwrap().params[0]
        .ty
        .slot
        .is_none());
    let mut direct = parse(source);
    bind_input(
        &mut direct,
        &FinalizingCodec,
        "take",
        "p",
        &Input::constructor("pair", [Input::Leaf, Input::Leaf]),
        &Extra::default(),
    )
    .unwrap();
    bind_output(
        &mut direct,
        &FinalizingCodec,
        "take",
        &Output::fields([]),
        &Extra::default(),
    )
    .unwrap();
    let declarations = Declarations::new(&direct, vec![]).unwrap();
    let decl = FunctionDecl::new(syn::parse_quote!(take)).expand_param(
        "p",
        ExpandParamDecl::new(syn::parse_quote!(Pair)).variant(fun!(pair)),
    );
    let mut flat = parse(source);
    assert_eq!(
        declarations
            .fill(
                &mut flat,
                &FinalizingCodec,
                &decl,
                &Extra {
                    name: "fail",
                    ..Extra::default()
                }
            )
            .unwrap_err(),
        "finalization failed"
    );
    assert!(flat.function("take").unwrap().params[0].ty.slot.is_none());
    assert!(flat.function("take").unwrap().ret.slot.is_none());
    declarations
        .fill(&mut flat, &FinalizingCodec, &decl, &Extra::default())
        .unwrap();
    assert_eq!(
        prebindgen_flat::emit_call(flat.function("take").unwrap(), &emission_context())
            .unwrap()
            .to_string(),
        prebindgen_flat::emit_call(direct.function("take").unwrap(), &emission_context())
            .unwrap()
            .to_string()
    );
}

fn point_shapes() -> (Bindings<TestWire>, Representations<()>, RepresentationId) {
    let flat = parse("pub struct Point { pub x:i64, pub y:i64 } pub fn point()->Point {}");
    let id = RepresentationId("PointFields".into());
    let mut shapes = Representations::default();
    shapes
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
                data: (),
            },
        )
        .unwrap();
    shapes.set_canonical(&id).unwrap();
    (flat, shapes, id)
}

#[test]
fn preparation_checks_representation_fields_references_and_mappings() {
    let (mut flat, mut shapes, id) = point_shapes();
    shapes.validate(&flat).unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "point",
        &Output::fields([Output::Leaf, Output::Leaf]),
        &Extra::default(),
    )
    .unwrap();
    let wires = &flat
        .function("point")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap()
        .wires;
    let ty = syn::parse_quote!(crate::Point);
    shapes
        .validate_mapping(&ty, &id, wires.iter().map(|w| (&w.name, &w.kind.component)))
        .unwrap();
    assert!(shapes
        .validate_mapping(
            &ty,
            &id,
            wires[..1].iter().map(|w| (&w.name, &w.kind.component))
        )
        .unwrap_err()
        .contains("has no wire"));
    let mut bad_wires = wires.clone();
    bad_wires[0].kind.component = ComponentPath(vec![ComponentSegment::Named("missing".into())]);
    assert!(shapes
        .validate_mapping(
            &ty,
            &id,
            bad_wires.iter().map(|w| (&w.name, &w.kind.component))
        )
        .unwrap_err()
        .contains("no component"));
    assert!(shapes
        .validate_mapping(
            &syn::parse_quote!(i64),
            &id,
            wires.iter().map(|w| (&w.name, &w.kind.component))
        )
        .unwrap_err()
        .contains("does not match"));
    let original = shapes.get(&id).unwrap().clone();
    let mut bad = original.clone();
    bad.components[0].source_fields = Some(vec!["missing".into()]);
    shapes.insert(id.clone(), bad).unwrap();
    assert!(shapes
        .validate(&flat)
        .unwrap_err()
        .contains("unknown source field"));
    let mut bad = original.clone();
    bad.components[0].representation = Some(RepresentationId("MissingType".into()));
    shapes.insert(id.clone(), bad).unwrap();
    assert!(shapes
        .validate(&flat)
        .unwrap_err()
        .contains("unknown representation"));
    let mut bad = original.clone();
    bad.components.push(bad.components[0].clone());
    assert!(shapes
        .insert(id.clone(), bad)
        .unwrap_err()
        .contains("duplicate component"));
    let mut bad = original;
    bad.components[0].representation = Some(id.clone());
    shapes.insert(id, bad).unwrap();
    assert!(shapes
        .validate(&flat)
        .unwrap_err()
        .contains("incompatible source type"));
}

#[test]
fn generation_needs_no_preparation_registry_or_record_lookup() {
    let (mut flat, mut shapes, id) = point_shapes();
    shapes.validate(&flat).unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "point",
        &Output::fields([Output::Leaf, Output::Leaf]),
        &Extra::default(),
    )
    .unwrap();
    let before = prebindgen_flat::emit_call(flat.function("point").unwrap(), &emission_context())
        .unwrap()
        .to_string();
    let handle = RepresentationId("PointHandle".into());
    shapes
        .insert(
            handle.clone(),
            Representation {
                source_type: syn::parse_quote!(crate::Point),
                components: vec![],
                data: (),
            },
        )
        .unwrap();
    shapes.set_canonical(&handle).unwrap();
    assert_ne!(
        shapes.canonical(&syn::parse_quote!(crate::Point)),
        Some(&id)
    );
    drop(shapes);
    flat.records.clear();
    let call = flat.function("point").unwrap();
    assert_eq!(call.outputs(&emission_context()).unwrap().len(), 2);
    assert_eq!(
        prebindgen_flat::emit_call(call, &emission_context())
            .unwrap()
            .to_string(),
        before
    );
    assert_eq!(
        call.outputs(&emission_context())
            .unwrap()
            .next()
            .unwrap()
            .wire
            .kind
            .component
            .0,
        [ComponentSegment::Field("x".into())]
    );
}

fn emission_context() -> syn::Ident {
    syn::parse_quote!(__result)
}
