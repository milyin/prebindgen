use super::*;
#[derive(Clone, Debug)]
enum TestGroup {
    Maybe,
    Record,
    Member(syn::Ident),
}
type Bindings<W> = super::Bindings<W, (), TestGroup>;
type InputPlan<W> = prebindgen_conversions::InputPlan<W, (), TestGroup>;
type OutputPlan<W> = prebindgen_conversions::OutputPlan<W, (), TestGroup>;
use prebindgen_conversions::staged::StagedFunction;
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
impl prebindgen_flat::WireKind for TestWire {
    fn rust_type(&self) -> syn::Type {
        syn::parse_str(&self.ty).unwrap()
    }
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
    fn result_error(&self, _: &Type, message: TokenStream, _: &Extra) -> Result<TokenStream> {
        Ok(message)
    }
    fn niche(&self, _: &TestWire) -> Option<()> {
        None
    }
    fn set_niche(&self, _: &mut TestWire, _: Option<()>) {}
    type Niche = ();
    type Group = TestGroup;
    fn optional_group(&self, _: syn::Ident, _: syn::Ident, _: Option<()>) -> TestGroup {
        TestGroup::Maybe
    }
    fn field_group(&self, name: syn::Ident) -> TestGroup {
        TestGroup::Member(name)
    }
    fn fields_group(&self) -> TestGroup {
        TestGroup::Record
    }
    type Extra = Extra;
    type Wire = TestWire;
    type Context = ();
    type Input = InputPlan<TestWire>;
    type Output = OutputPlan<TestWire>;
    fn prefix_component(&self, wire: &mut TestWire, segment: ComponentSegment) {
        wire.component.prepend(segment);
    }
    fn absent_value(&self, _: &TestWire, _: Option<&()>, _: bool) -> Result<TokenStream> {
        Ok(quote!(Default::default()))
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
            fallible: false,
            context: Default::default(),
            acknowledgement: None,
            layout: vec![Wire {
                name: w.clone(),
                kind: wire(Kind::Token, ty, extra),
            }]
            .into(),
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
            fallible: false,
            context: Default::default(),
            delivery: Default::default(),
            layout: vec![Wire {
                name: w.clone(),
                kind: wire(Kind::Token, ty, extra),
            }]
            .into(),
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
    assert_eq!(x.layout.len(), 3);
    assert_eq!(y.layout.len(), 1);
    assert!(x.layout.iter().all(|w| w.kind.name == "x"));
    assert_eq!(y.layout[0].kind.name, "y");
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
    assert!(o.layout[0].kind.kind == Kind::Token);
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
            .layout
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
    assert_eq!(x.layout.len(), 3);
    assert!(x.layout[2].kind.kind == Kind::Token);
    assert!(x.layout.iter().all(|w| w.kind.name == "x"));
    assert_eq!(y.layout.len(), 1);
    assert_eq!(y.layout[0].kind.name, "override");
    assert_eq!(f.ret.slot.as_ref().unwrap().layout[0].kind.name, "result");
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
        .flat_map(|p| &p.ty.slot.as_ref().unwrap().layout)
        .chain(&f.ret.slot.as_ref().unwrap().layout)
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
    assert!(output.layout[0].kind.kind == Kind::Value(Value::Text));
    assert!(output.layout[1].kind.kind == Kind::Value(Value::Bytes));
    assert_eq!(output.layout[0].kind.ty, "String");
    assert_eq!(output.layout[1].kind.ty, "Vec < u8 >");
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
    assert_eq!(input.layout[0].kind.kind, scalar);
    assert_eq!(output.layout[0].kind.kind, scalar);
    assert_eq!(input.layout[0].kind.name, "scalar");
    assert_eq!(input.layout[0].kind.ty, "i64");
    assert_eq!(output.layout[0].kind.name, "scalar");
    assert_eq!(output.layout[0].kind.ty, "i64");
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
    let foreign = |wires: &WireLayout<TestWire, TestGroup>| {
        wires
            .iter()
            .filter_map(|w| match &w.kind.kind {
                Kind::Token => Some((w.kind.name, w.kind.ty.clone())),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        foreign(&f.params[0].ty.slot.as_ref().unwrap().layout),
        vec![
            ("x", "crate :: Payload".into()),
            ("x", "crate :: Key".into())
        ]
    );
    assert_eq!(
        foreign(&f.params[1].ty.slot.as_ref().unwrap().layout),
        vec![("y", "crate :: Key".into())]
    );
    assert_eq!(
        foreign(&f.ret.slot.as_ref().unwrap().layout),
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
        call.outputs(&(), &source_binding())
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
        fn niche(&self, _: &TestWire) -> Option<()> {
            None
        }
        fn set_niche(&self, _: &mut TestWire, _: Option<()>) {}
        type Niche = ();
        type Group = TestGroup;
        fn optional_group(&self, _: syn::Ident, _: syn::Ident, _: Option<()>) -> TestGroup {
            TestGroup::Maybe
        }
        fn field_group(&self, name: syn::Ident) -> TestGroup {
            TestGroup::Member(name)
        }
        fn fields_group(&self) -> TestGroup {
            TestGroup::Record
        }
        type Extra = Extra;
        type Wire = TestWire;
        type Context = ();
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
            assert_eq!(plan.layout.len(), 2);
            Ok(())
        }
        fn finish_output(&self, plan: &mut OutputPlan<TestWire>, extra: &Extra) -> Result<()> {
            if extra.name == "fail" {
                return Err("finalization failed".into());
            }
            assert!(plan.delivery.statements.is_empty());
            assert!(plan.layout.is_empty());
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
        prebindgen_conversions::staged::emit_call(
            flat.function("take").unwrap(),
            &(),
            &source_binding(),
            &syn::parse_quote!(_)
        )
        .unwrap()
        .to_string(),
        prebindgen_conversions::staged::emit_call(
            direct.function("take").unwrap(),
            &(),
            &source_binding(),
            &syn::parse_quote!(_)
        )
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
        .layout;
    let ty = syn::parse_quote!(crate::Point);
    shapes
        .validate_mapping(&ty, &id, wires.iter().map(|w| (&w.name, &w.kind.component)))
        .unwrap();
    assert!(shapes
        .validate_mapping(
            &ty,
            &id,
            wires.iter().take(1).map(|w| (&w.name, &w.kind.component))
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
    let before = prebindgen_conversions::staged::emit_call(
        flat.function("point").unwrap(),
        &(),
        &source_binding(),
        &syn::parse_quote!(_),
    )
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
    assert_eq!(call.outputs(&(), &source_binding()).unwrap().len(), 2);
    assert_eq!(
        prebindgen_conversions::staged::emit_call(
            call,
            &(),
            &source_binding(),
            &syn::parse_quote!(_)
        )
        .unwrap()
        .to_string(),
        before
    );
    assert_eq!(
        call.outputs(&(), &source_binding())
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

fn source_binding() -> syn::Ident {
    syn::parse_quote!(__result)
}

#[test]
fn shared_record_input_preserves_ownership_phases_and_rejects_conflicting_acknowledgements() {
    use prebindgen_conversions::{Access, Resource};
    let make = |index: usize, ack: TokenStream| {
        let name = format_ident!("field{index}");
        let mut child = TEST_CODEC
            .input_leaf(
                &syn::parse_quote!(i64),
                &Extra::default(),
                format_ident!("wire{index}"),
                format_ident!("value{index}"),
                &Condition::default(),
            )
            .unwrap();
        child.validation = quote!(validate(#index););
        child.acquire = quote!(acquire(#index););
        child.acknowledgement = Some(ack);
        child.resources.push(Resource {
            identity: quote!(#index),
            access: Access::Move,
            active: quote!(selected),
        });
        Ok((name, child))
    };
    let plan = record_input(
        &TEST_CODEC,
        &syn::parse_quote!(Pair),
        format_ident!("pair"),
        [make(0, quote!(ack())), make(1, quote!(ack()))],
    )
    .unwrap();
    assert_eq!(
        plan.validation.to_string(),
        quote!(validate(0usize); validate(1usize);).to_string()
    );
    assert_eq!(
        plan.acquire.to_string(),
        quote!(acquire(0usize); acquire(1usize);).to_string()
    );
    assert_eq!(plan.resources.len(), 2);
    assert!(plan
        .resources
        .iter()
        .all(|r| r.access == Access::Move && r.active.to_string() == "selected"));
    assert_eq!(
        plan.acknowledgement.unwrap().to_string(),
        quote!(ack()).to_string()
    );
    assert_eq!(
        plan.layout[1].kind.component,
        ComponentPath(vec![ComponentSegment::Field("field1".into())])
    );
    assert!(record_input(
        &TEST_CODEC,
        &syn::parse_quote!(Pair),
        format_ident!("pair"),
        [make(0, quote!(ack())), make(1, quote!(different()))]
    )
    .unwrap_err()
    .contains("conflicting input acknowledgements"));
}

#[test]
fn shared_record_output_evaluates_once_preserves_commit_and_rejects_early_delivery() {
    let make = |early_delivery| {
        record_output(
            &TEST_CODEC,
            format_ident!("record"),
            quote!(next_record()),
            [format_ident!("x"), format_ident!("y")],
            |i, field| {
                let mut child = TEST_CODEC.output_leaf(
                    &syn::parse_quote!(i64),
                    &Extra::default(),
                    format_ident!("wire{i}"),
                    field,
                    true,
                )?;
                child.commit = quote!(commit(#i););
                if early_delivery {
                    child.delivery.statements = quote!(deliver());
                }
                Ok(child)
            },
        )
    };
    let plan = make(false).unwrap();
    assert_eq!(
        plan.statements.to_string().matches("next_record").count(),
        1
    );
    assert_eq!(
        plan.commit.to_string(),
        quote!(commit(0usize); commit(1usize);).to_string()
    );
    assert_eq!(plan.values.len(), 2);
    assert_eq!(
        plan.layout[1].kind.component,
        ComponentPath(vec![ComponentSegment::Field("y".into())])
    );
    assert!(make(true).unwrap_err().contains("finish_output"));
}

#[test]
fn shared_empty_record_output_evaluates_without_an_unused_local() {
    let plan = record_output(
        &TEST_CODEC,
        format_ident!("record"),
        quote!(next_record()),
        [],
        |_, _| unreachable!("empty records have no field converters"),
    )
    .unwrap();
    assert_eq!(
        plan.statements.to_string(),
        quote!(let _ = &(next_record());).to_string()
    );
    assert!(plan.layout.is_empty());
}

#[test]
fn owned_record_cleanup_covers_partial_conversion_and_failed_delivery() {
    let plan = record_output_owned(
        &TEST_CODEC,
        format_ident!("record"),
        quote!(make_record(drops.clone())),
        [format_ident!("first"), format_ident!("second")],
        |index, field| {
            let local = format_ident!("box{index}");
            Ok(OutputPlan {
                fallible: false,
                context: Default::default(),
                delivery: Default::default(),
                layout: Default::default(),
                values: Vec::new(),
                statements: quote! {
                    if mode == #index + 1 { return Err(()); }
                    let #local = Box::new(#field);
                },
                // Test delivery reclaims each committed allocation immediately.
                commit: quote!(unsafe { drop(Box::from_raw(Box::into_raw(#local))); }),
            })
        },
    )
    .unwrap();
    let prepare = plan.statements;
    let commit = plan.commit;
    let source = quote! {
        use std::{cell::Cell, rc::Rc};
        struct Resource(Rc<Cell<usize>>);
        impl Drop for Resource { fn drop(&mut self) { self.0.set(self.0.get() + 1); } }
        struct Record { first: Resource, second: Resource }
        fn make_record(drops: Rc<Cell<usize>>) -> Record {
            Record { first: Resource(drops.clone()), second: Resource(drops) }
        }
        fn convert(drops: Rc<Cell<usize>>, mode: usize) -> Result<(), ()> {
            #prepare
            if mode == 3 { return Err(()); } // delivery fails before commit
            #commit
            Ok(())
        }
        fn main() {
            for mode in 0..4 {
                let drops = Rc::new(Cell::new(0));
                assert_eq!(convert(drops.clone(), mode).is_ok(), mode == 0);
                assert_eq!(drops.get(), 2);
            }
        }
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("owned.rs");
    let exe = dir.path().join("owned");
    std::fs::write(&file, source.to_string()).unwrap();
    let compiled = std::process::Command::new("rustc")
        .arg(&file)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    assert!(std::process::Command::new(exe).status().unwrap().success());
}

#[test]
fn consuming_field_accessors_validate_signatures_and_order_observers() {
    let mut flat = parse("pub struct Pair { pub first:i64, pub second:i64 } pub fn make()->Pair {} pub fn into_pair(value:Pair)->Pair {} pub fn copy_pair(value:&Pair)->Pair {} pub fn count(value:&Pair)->i64 {}");
    let declarations = Declarations::new(&flat, vec![]).unwrap();
    declarations
        .fill(
            &mut flat,
            &TEST_CODEC,
            &fun!(make).expand_return(
                expand_return!(Pair)
                    .fields_into(
                        fields!(into_pair).field("first", expand_return!(i64).field_self_ref()),
                    )
                    .field(fun!(count)),
            ),
            &Extra::default(),
        )
        .unwrap();
    let statements = flat
        .function("make")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap()
        .statements
        .to_string();
    assert!(statements.find("count").unwrap() < statements.find("into_pair").unwrap());
    let error = declarations
        .fill(
            &mut flat,
            &TEST_CODEC,
            &fun!(make).expand_return(expand_return!(Pair).fields_into(fields!(copy_pair))),
            &Extra::default(),
        )
        .unwrap_err();
    assert!(
        error.contains("must take exactly `crate :: Pair`"),
        "{error}"
    );
    let error = declarations
        .fill(
            &mut flat,
            &TEST_CODEC,
            &fun!(make).expand_return(expand_return!(Pair).fields(fields!(into_pair))),
            &Extra::default(),
        )
        .unwrap_err();
    assert!(
        error.contains("must take exactly `& crate :: Pair`"),
        "{error}"
    );
}

#[test]
fn optional_output_drops_prepared_owners_on_failure_and_skips_absent_payloads() {
    let child = record_output_owned(
        &TEST_CODEC,
        format_ident!("record"),
        quote!(payload),
        [format_ident!("first"), format_ident!("second")],
        |index, field| {
            let local = format_ident!("box{index}");
            Ok(OutputPlan {
                fallible: false,
                context: Default::default(),
                delivery: Default::default(),
                layout: vec![].into(),
                values: vec![],
                statements: quote!(if mode==#index+1{return Err(());} let #local=Box::new(#field);),
                commit: quote!(drop(#local);),
            })
        },
    )
    .unwrap();
    let plan = optional_output(
        &TEST_CODEC,
        &Extra::default(),
        format_ident!("present"),
        format_ident!("payload"),
        quote!(source),
        false,
        child,
    )
    .unwrap();
    let prepare = plan.statements;
    let commit = plan.commit;
    let source = quote! {
        use std::{cell::Cell,rc::Rc};
        struct R(Rc<Cell<usize>>);impl Drop for R{fn drop(&mut self){self.0.set(self.0.get()+1);}}
        struct Record{first:R,second:R}
        fn convert(drops:Rc<Cell<usize>>,some:bool,mode:usize)->Result<(),()> {
            let source=some.then(||Record{first:R(drops.clone()),second:R(drops.clone())});
            #prepare
            if mode==3{return Err(());}
            #commit
            Ok(())
        }
        fn main(){for some in [false,true]{for mode in 0..4 {let drops=Rc::new(Cell::new(0));let result=convert(drops.clone(),some,mode);assert_eq!(result.is_ok(),mode==0 || (!some && mode!=3));assert_eq!(drops.get(),if some{2}else{0});}}}
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("option.rs");
    let exe = dir.path().join("option");
    std::fs::write(&file, source.to_string()).unwrap();
    let result = std::process::Command::new("rustc")
        .arg(&file)
        .arg("-o")
        .arg(&exe)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(std::process::Command::new(exe).status().unwrap().success());
}

#[test]
fn field_groups_preserve_empty_and_multiwire_fields_in_both_directions() {
    use prebindgen_flat::WireNode;
    let mut flat = parse("pub struct Empty {} pub struct Pair { pub x: i32, pub y: i32 } pub struct Record { pub maybe: Option<i32>, pub empty: Empty, pub pair: Pair } pub fn echo(value: Record) -> Record { value }");
    let input = Input::Fields(vec![
        Input::Optional(Box::new(Input::Leaf)),
        Input::Fields(vec![]),
        Input::Fields(vec![Input::Leaf, Input::Leaf]),
    ]);
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "echo",
        "value",
        &input,
        &Extra::default(),
    )
    .unwrap();
    fn check(layout: &WireLayout<TestWire, TestGroup>) {
        layout.validate().unwrap();
        assert_eq!(layout.len(), 4);
        let [WireNode::Group { metadata, children }] = layout.nodes() else {
            panic!()
        };
        assert!(matches!(metadata, TestGroup::Record));
        let mut names = Vec::new();
        let mut widths = Vec::new();
        for node in children.nodes() {
            let WireNode::Group { metadata, children } = node else {
                panic!()
            };
            let TestGroup::Member(name) = metadata else {
                panic!()
            };
            names.push(name.to_string());
            widths.push(children.len());
        }
        assert_eq!(names, ["maybe", "empty", "pair"]);
        assert_eq!(widths, [2, 0, 2]);
        for (i, wire) in layout.iter().enumerate() {
            assert!(std::ptr::eq(wire, layout.get(i).unwrap()));
        }
    }
    check(
        &flat.function("echo").unwrap().params[0]
            .ty
            .slot
            .as_ref()
            .unwrap()
            .layout,
    );
    for owned in [false, true] {
        let fields = vec![
            Output::Optional(Box::new(Output::Leaf)),
            Output::Fields(vec![]),
            Output::Fields(vec![Output::Leaf, Output::Leaf]),
        ];
        let output = if owned {
            Output::OwnedFields(fields)
        } else {
            Output::Fields(fields)
        };
        bind_output(&mut flat, &TEST_CODEC, "echo", &output, &Extra::default()).unwrap();
        let plan = flat.function("echo").unwrap().ret.slot.as_ref().unwrap();
        check(&plan.layout);
        assert_eq!(plan.values.len(), 4);
    }
}

#[test]
fn record_helpers_check_source_field_names_without_reading_group_metadata() {
    let names = || [syn::parse_quote!(x), syn::parse_quote!(r#x)];
    let fields = names().into_iter().enumerate().map(|(i, name)| {
        TEST_CODEC
            .input_leaf(
                &syn::parse_quote!(i64),
                &Extra::default(),
                format_ident!("w{i}"),
                format_ident!("v{i}"),
                &Condition::default(),
            )
            .map(|plan| (name, plan))
    });
    let error = record_input(
        &TEST_CODEC,
        &syn::parse_quote!(Record),
        syn::parse_quote!(record),
        fields,
    )
    .unwrap_err();
    assert!(error.contains("duplicate field"));
    for owned in [false, true] {
        let convert = |i, value| {
            TEST_CODEC.output_leaf(
                &syn::parse_quote!(i64),
                &Extra::default(),
                format_ident!("w{i}"),
                value,
                !owned,
            )
        };
        let result = if owned {
            record_output_owned(
                &TEST_CODEC,
                syn::parse_quote!(record),
                quote!(source),
                names(),
                convert,
            )
        } else {
            record_output(
                &TEST_CODEC,
                syn::parse_quote!(record),
                quote!(source),
                names(),
                convert,
            )
        };
        assert!(result.unwrap_err().contains("duplicate field"));
    }
}

#[test]
fn result_defaults_overrides_and_borrowed_error_mappers_reuse_success_plans() {
    let mut flat = parse("pub type E=String; pub fn message(e:&E)->String{} pub fn other(e:E)->String{} pub fn echo(x:i64)->Result<Option<i64>,E>{}");
    let defaults = Declarations::new(
        &flat,
        vec![ErrorDecl::new(syn::parse_quote!(E), fun!(message)).into()],
    )
    .unwrap();
    defaults
        .fill(&mut flat, &TEST_CODEC, &fun!(echo), &Extra::default())
        .unwrap();
    let output = flat.function("echo").unwrap().ret.slot.as_ref().unwrap();
    assert_eq!(output.layout.len(), 2);
    let code = output.statements.to_string();
    assert!(code.contains(":: core :: result :: Result :: Ok"));
    assert!(code.contains("message (&"));
    assert!(code.find("match").unwrap() < code.find("message").unwrap());
    defaults
        .fill(
            &mut flat,
            &TEST_CODEC,
            &fun!(echo).on_error(fun!(other)),
            &Extra::default(),
        )
        .unwrap();
    let code = flat
        .function("echo")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap()
        .statements
        .to_string();
    assert!(code.contains("other (") && !code.contains("message"));
}

#[test]
fn invalid_result_policies_leave_all_function_slots_unchanged() {
    for mapper in [
        None,
        Some(fun!(wrong)),
        Some(fun!(message).name("renamed")),
        Some(fun!(message).on_error(fun!(message))),
    ] {
        let mut flat = parse("pub type E=String; pub fn message(e:E)->String{} pub fn wrong(e:&mut E)->String{} pub fn f(x:i64)->Result<i64,E>{}");
        let mut decl = fun!(f);
        if let Some(mapper) = mapper {
            decl = decl.on_error(mapper);
        }
        assert!(fill(&mut flat, &decl).is_err());
        let f = flat.function("f").unwrap();
        assert!(f.ret.slot.is_none() && f.params[0].ty.slot.is_none());
    }
    for source in [
        "pub type E=String;pub fn message(e:E)->i64{}pub fn f()->Result<i64,E>{}",
        "pub type E=String;pub fn message()->String{}pub fn f()->Result<i64,E>{}",
        "pub type E=String;pub fn message(e:E,x:i64)->String{}pub fn f()->Result<i64,E>{}",
        "pub type E=String;pub fn message(e:String)->String{}pub fn f()->Result<i64,E>{}",
        "pub type E=String;pub fn message(e:E)->String{}pub fn f()->i64{}",
    ] {
        let mut flat = parse(source);
        assert!(
            fill(&mut flat, &fun!(f).on_error(fun!(message))).is_err(),
            "{source}"
        );
    }
}

#[test]
fn result_routing_rejects_nested_and_borrowed_data_occurrences() {
    for source in [
        "pub fn f(x:Result<i64,String>){}",
        "pub fn f(x:Option<&Result<i64,String>>){}",
    ] {
        let mut flat = parse(source);
        assert!(fill(&mut flat, &fun!(f))
            .unwrap_err()
            .contains("output-only"));
    }
    for (source, recipe) in [
        (
            "pub fn f()->Option<Result<i64,String>>{}",
            Output::Optional(Box::new(Output::Leaf)),
        ),
        ("pub fn f()->&Result<i64,String>{}", Output::Leaf),
        (
            "pub struct R{pub value:Result<i64,String>}pub fn f()->R{}",
            Output::Fields(vec![Output::Leaf]),
        ),
    ] {
        let mut flat = parse(source);
        assert!(
            bind_output(&mut flat, &TEST_CODEC, "f", &recipe, &Extra::default())
                .unwrap_err()
                .contains("outermost")
        );
    }
    let mut flat =
        parse("pub fn message(e:String)->String{}pub fn f()->Result<Result<i64,String>,String>{}");
    assert!(fill(&mut flat, &fun!(f).on_error(fun!(message)))
        .unwrap_err()
        .contains("outermost"));
    for decl in [
        expand_return!(Result<i64,String>).field_self().into(),
        expand_param!(Result<i64,String>).variant_self().into(),
    ] {
        assert!(Declarations::new(&flat, vec![decl]).is_err());
    }
}

#[test]
fn result_error_defaults_are_explicit_unique_and_selected_with_helpers() {
    let flat = parse("pub type E=String;pub fn message(e:E)->String{}pub fn f()->Result<i64,E>{}");
    assert!(Declarations::new(
        &flat,
        vec![
            ErrorDecl::new(syn::parse_quote!(E), fun!(message)).into(),
            ErrorDecl::new(syn::parse_quote!(E), fun!(message)).into()
        ]
    )
    .is_err());
    let expansions = vec![ErrorDecl::new(syn::parse_quote!(E), fun!(message)).into()];
    let names = declarations::source_functions(&expansions, [&fun!(f).on_error(fun!(message))]);
    assert!(names.contains(&"f".into()) && names.contains(&"message".into()));
}
