use super::*;
use prebindgen_flat::InputConversion;
#[derive(Clone, Debug)]
enum TestGroup {
    Values,
    Maybe,
    Record,
    Member(syn::Ident),
}
struct TestAbi;
impl TestAbi {
    fn wires(
        node: &WireNode<TestWire, TestGroup, TestGroup>,
    ) -> std::vec::IntoIter<&Wire<TestWire>> {
        prebindgen_flat::wire_definitions(std::iter::once(node))
            .collect::<Vec<_>>()
            .into_iter()
    }
    fn wires_mut(
        node: &mut WireNode<TestWire, TestGroup, TestGroup>,
    ) -> std::vec::IntoIter<&mut Wire<TestWire>> {
        prebindgen_flat::wire_definitions_mut(std::iter::once(node))
            .collect::<Vec<_>>()
            .into_iter()
    }
    fn validate(node: &WireNode<TestWire, TestGroup, TestGroup>) -> Result<()> {
        fn check(node: &WireNode<TestWire, TestGroup, TestGroup>) -> Result<()> {
            match node {
                WireNode::Leaf(_) => {}
                WireNode::Chain { child, .. } => check(child)?,
                WireNode::Branch { meta, children, .. } => {
                    if matches!(meta, TestGroup::Maybe) && children.is_empty() {
                        return Err("selection has no alternatives".into());
                    }
                    for child in children {
                        check(child)?;
                    }
                }
            }
            Ok(())
        }
        check(node)?;
        prebindgen_conversions::validate_wire_names(Self::wires(node))
    }
}
type Bindings<W> = super::Bindings<W, (), TestGroup, TestGroup>;
type InputPlan<W> = prebindgen_conversions::InputPlan<W, (), TestGroup, TestGroup>;
type OutputPlan<W> = prebindgen_conversions::OutputPlan<W, (), TestGroup, TestGroup>;
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
#[allow(clippy::type_complexity)] // Keep the test codec metadata roles visible.
impl Codec for TestCodec {
    type Metadata = ();
    fn binding_failure_layout(
        &self,
        extra: &Extra,
    ) -> Result<Option<WireNode<TestWire, TestGroup, TestGroup>>> {
        Ok(
            (extra.name == "malformed_binding_failure").then(|| WireNode::Branch {
                controls: vec![],
                children: vec![],
                meta: TestGroup::Maybe,
            }),
        )
    }
    fn result_error(&self, _: &Type, message: TokenStream, _: &Extra) -> Result<TokenStream> {
        Ok(message)
    }
    fn niche(&self, _: &TestWire) -> Option<()> {
        None
    }
    fn set_niche(&self, _: &mut TestWire, _: Option<()>) {}
    type Niche = ();
    type Chain = TestGroup;
    type Branch = TestGroup;
    fn group_metadata(&self) -> TestGroup {
        TestGroup::Values
    }
    fn optional_niche(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
    ) -> Option<NicheReservation<Self::Niche>> {
        select_optional_niche(self, TestAbi::wires(payload))
    }
    fn optional_presence(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: &NicheReservation<Self::Niche>,
    ) -> Result<TokenStream> {
        optional_presence(self, TestAbi::wires(payload), reservation)
    }
    fn absent_output_values(
        &self,
        payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<Vec<(syn::Ident, TokenStream)>> {
        absent_output_values(self, TestAbi::wires(payload), reservation)
    }
    fn retire_niches(
        &self,
        payload: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        reservation: Option<&NicheReservation<Self::Niche>>,
    ) -> Result<()> {
        retire_optional_niches(self, TestAbi::wires_mut(payload), reservation)
    }
    fn validate_input(&self, plan: &CodecInput<Self>) -> Result<()> {
        TestAbi::validate(&plan.node)
    }
    fn validate_output(&self, plan: &CodecOutput<Self>) -> Result<()> {
        for node in std::iter::once(&plan.outcomes.success)
            .chain(plan.outcomes.source_error.iter())
            .chain(plan.outcomes.binding_failure.iter())
        {
            TestAbi::validate(node)?;
        }
        prebindgen_conversions::match_output_fields(
            TestAbi::wires(&plan.outcomes.success),
            &plan.values,
        )?;
        Ok(())
    }
    fn prefix_layout(
        &self,
        layout: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        segment: ComponentSegment,
    ) {
        for w in TestAbi::wires_mut(layout) {
            w.kind.component.prepend(segment.clone());
        }
    }
    fn optional_metadata(
        &self,
        _: syn::Ident,
        _: syn::Ident,
        _: Option<()>,
        controls: &[Wire<TestWire>],
    ) -> Result<TestGroup> {
        if controls
            .iter()
            .any(|wire| wire.kind.name == "reject_optional_metadata")
        {
            return Err("optional metadata rejected".into());
        }
        Ok(TestGroup::Maybe)
    }

    fn alternatives_metadata(&self, selector: &Wire<TestWire>) -> Result<TestGroup> {
        if selector.kind.name == "reject_alternative_metadata" {
            return Err("alternative metadata rejected".into());
        }
        Ok(TestGroup::Maybe)
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
            node: prebindgen_flat::WireNode::Leaf(vec![Wire {
                name: w.clone(),
                kind: wire(Kind::Token, ty, extra),
            }]),
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
            outcomes: prebindgen_flat::Outcomes {
                success: prebindgen_flat::WireNode::Leaf(vec![Wire {
                    name: w.clone(),
                    kind: wire(Kind::Token, ty, extra),
                }]),
                source_error: None,
                binding_failure: None,
            },
            statements: quote!(let #w=send_token(#v);),
            values: vec![(w.clone(), quote!(#w))],
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
    Declarations::new(flat, vec![], vec![], vec![])?.fill(
        flat,
        &TEST_CODEC,
        decl,
        &Extra::default(),
    )
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
    assert_eq!(TestAbi::wires(&x.node).len(), 3);
    assert_eq!(TestAbi::wires(&y.node).len(), 1);
    assert!(TestAbi::wires(&x.node).all(|w| w.kind.name == "x"));
    assert_eq!(TestAbi::wires(&y.node).next().unwrap().kind.name, "y");
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
    assert!(
        TestAbi::wires(&o.outcomes.success)
            .next()
            .unwrap()
            .kind
            .kind
            == Kind::Token
    );
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
        let function = fun!(read).expand_return(
            expand_return!(Payload).fields(fields!(parts).field_override(field, output)),
        );
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
        vec![],
        vec![],
    )
    .unwrap();
    declarations
        .fill(&mut flat, &TEST_CODEC, &fun!(foo), &extra("defaults"))
        .unwrap();
    assert_eq!(
        TestAbi::wires(
            &flat.function("foo").unwrap().params[1]
                .ty
                .slot
                .as_ref()
                .unwrap()
                .node
        )
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
    assert_eq!(TestAbi::wires(&x.node).len(), 3);
    assert!(TestAbi::wires(&x.node).nth(2).unwrap().kind.kind == Kind::Token);
    assert!(TestAbi::wires(&x.node).all(|w| w.kind.name == "x"));
    assert_eq!(TestAbi::wires(&y.node).len(), 1);
    assert_eq!(
        TestAbi::wires(&y.node).next().unwrap().kind.name,
        "override"
    );
    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(
            &f.ret.slot.as_ref().unwrap().outcomes.success
        ))
        .next()
        .expect("logical wire index out of range")
        .kind
        .name,
        "result"
    );
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
        .flat_map(|p| {
            prebindgen_flat::wire_definitions(std::iter::once(&p.ty.slot.as_ref().unwrap().node))
        })
        .chain(prebindgen_flat::wire_definitions(std::iter::once(
            &f.ret.slot.as_ref().unwrap().outcomes.success,
        )))
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
        vec![],
        vec![expand_return!(Key).field(fun!(key_text)).into()],
        vec![],
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
    assert!(
        TestAbi::wires(&output.outcomes.success)
            .next()
            .unwrap()
            .kind
            .kind
            == Kind::Value(Value::Text)
    );
    assert!(
        TestAbi::wires(&output.outcomes.success)
            .nth(1)
            .unwrap()
            .kind
            .kind
            == Kind::Value(Value::Bytes)
    );
    assert_eq!(
        TestAbi::wires(&output.outcomes.success)
            .next()
            .unwrap()
            .kind
            .ty,
        "String"
    );
    assert_eq!(
        TestAbi::wires(&output.outcomes.success)
            .nth(1)
            .unwrap()
            .kind
            .ty,
        "Vec < u8 >"
    );
}
#[test]
fn explicit_scalar_identity_is_a_value_conversion() {
    let mut flat = parse("pub fn f(x:i64)->i64{}");
    let defaults = Declarations::new(
        &flat,
        vec![expand_param!(i64).variant_self().into()],
        vec![expand_return!(i64).field_self().into()],
        vec![],
    )
    .unwrap();
    defaults
        .fill(&mut flat, &TEST_CODEC, &fun!(f), &extra("scalar"))
        .unwrap();
    let f = flat.function("f").unwrap();
    let input = f.params[0].ty.slot.as_ref().unwrap();
    let output = f.ret.slot.as_ref().unwrap();
    let scalar = Kind::Value(Value::Scalar(prebindgen_flat::Primitive::I64));
    assert_eq!(
        TestAbi::wires(&input.node).next().unwrap().kind.kind,
        scalar
    );
    assert_eq!(
        TestAbi::wires(&output.outcomes.success)
            .next()
            .unwrap()
            .kind
            .kind,
        scalar
    );
    assert_eq!(
        TestAbi::wires(&input.node).next().unwrap().kind.name,
        "scalar"
    );
    assert_eq!(TestAbi::wires(&input.node).next().unwrap().kind.ty, "i64");
    assert_eq!(
        TestAbi::wires(&output.outcomes.success)
            .next()
            .unwrap()
            .kind
            .name,
        "scalar"
    );
    assert_eq!(
        TestAbi::wires(&output.outcomes.success)
            .next()
            .unwrap()
            .kind
            .ty,
        "i64"
    );
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
    let foreign = |wires: &prebindgen_flat::WireNode<TestWire, TestGroup, TestGroup>| {
        prebindgen_flat::wire_definitions(std::iter::once(wires))
            .filter_map(|w| match &w.kind.kind {
                Kind::Token => Some((w.kind.name, w.kind.ty.clone())),
                _ => None,
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        foreign(&f.params[0].ty.slot.as_ref().unwrap().node),
        vec![
            ("x", "crate :: Payload".into()),
            ("x", "crate :: Key".into())
        ]
    );
    assert_eq!(
        foreign(&f.params[1].ty.slot.as_ref().unwrap().node),
        vec![("y", "crate :: Key".into())]
    );
    assert_eq!(
        foreign(&f.ret.slot.as_ref().unwrap().outcomes.success),
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
        test_return_packet(call, &(), &source_binding())
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
        type Metadata = ();
        fn niche(&self, _: &TestWire) -> Option<()> {
            None
        }
        fn set_niche(&self, _: &mut TestWire, _: Option<()>) {}
        type Niche = ();
        type Chain = TestGroup;
        type Branch = TestGroup;
        fn group_metadata(&self) -> TestGroup {
            TestGroup::Values
        }
        fn optional_niche(
            &self,
            payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
        ) -> Option<NicheReservation<Self::Niche>> {
            select_optional_niche(self, TestAbi::wires(payload))
        }
        fn optional_presence(
            &self,
            payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
            reservation: &NicheReservation<Self::Niche>,
        ) -> Result<TokenStream> {
            optional_presence(self, TestAbi::wires(payload), reservation)
        }
        fn absent_output_values(
            &self,
            payload: &prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
            reservation: Option<&NicheReservation<Self::Niche>>,
        ) -> Result<Vec<(syn::Ident, TokenStream)>> {
            absent_output_values(self, TestAbi::wires(payload), reservation)
        }
        fn retire_niches(
            &self,
            payload: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
            reservation: Option<&NicheReservation<Self::Niche>>,
        ) -> Result<()> {
            retire_optional_niches(self, TestAbi::wires_mut(payload), reservation)
        }
        fn validate_input(&self, plan: &CodecInput<Self>) -> Result<()> {
            TestAbi::validate(&plan.node)
        }
        fn validate_output(&self, plan: &CodecOutput<Self>) -> Result<()> {
            for node in std::iter::once(&plan.outcomes.success)
                .chain(plan.outcomes.source_error.iter())
                .chain(plan.outcomes.binding_failure.iter())
            {
                TestAbi::validate(node)?;
            }
            prebindgen_conversions::match_output_fields(
                TestAbi::wires(&plan.outcomes.success),
                &plan.values,
            )?;
            Ok(())
        }
        fn prefix_layout(
            &self,
            layout: &mut prebindgen_flat::WireNode<Self::Wire, Self::Chain, Self::Branch>,
            segment: ComponentSegment,
        ) {
            for w in TestAbi::wires_mut(layout) {
                w.kind.component.prepend(segment.clone());
            }
        }
        fn optional_metadata(
            &self,
            _: syn::Ident,
            _: syn::Ident,
            _: Option<()>,
            _controls: &[Wire<TestWire>],
        ) -> Result<TestGroup> {
            Ok(TestGroup::Maybe)
        }

        fn alternatives_metadata(&self, _selector: &Wire<TestWire>) -> Result<TestGroup> {
            Ok(TestGroup::Maybe)
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
            assert_eq!(TestAbi::wires(&plan.node).len(), 2);
            assert_eq!(
                prebindgen_flat::wire_definitions(std::iter::once(&plan.node)).count(),
                2
            );
            plan.node = prebindgen_flat::WireNode::Branch {
                controls: vec![],
                meta: TestGroup::Record,
                children: vec![std::mem::take(&mut plan.node)],
            };
            Ok(())
        }
        fn finish_output(&self, plan: &mut OutputPlan<TestWire>, extra: &Extra) -> Result<()> {
            if extra.name == "fail" {
                return Err("finalization failed".into());
            }
            assert!(plan.delivery.statements.is_empty());
            assert!(TestAbi::wires(&plan.outcomes.success).len() == 0);
            assert!(matches!(
                &plan.outcomes.success,
                prebindgen_flat::WireNode::Branch { .. }
            ));
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
    assert!(matches!(
        &direct
            .function("take")
            .unwrap()
            .input_conversions()
            .unwrap()[0]
            .node,
        prebindgen_flat::WireNode::Branch {
            meta: TestGroup::Record,
            ..
        }
    ));
    bind_output(
        &mut direct,
        &FinalizingCodec,
        "take",
        &Output::fields([]),
        &Extra::default(),
    )
    .unwrap();
    let declarations = Declarations::new(&direct, vec![], vec![], vec![]).unwrap();
    let decl = FunctionDecl::new(syn::parse_quote!(take)).input(
        "p",
        Input::for_type(syn::parse_quote!(Pair), Input::Selector(vec![])).variant("pair"),
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
            &syn::parse_quote!(_),
            &test_parameter_names(flat.function("take").unwrap())
        )
        .unwrap()
        .to_string(),
        prebindgen_conversions::staged::emit_call(
            direct.function("take").unwrap(),
            &(),
            &source_binding(),
            &syn::parse_quote!(_),
            &test_parameter_names(direct.function("take").unwrap())
        )
        .unwrap()
        .to_string()
    );
}

#[test]
fn generation_needs_no_preparation_registry_or_record_lookup() {
    let mut flat = parse("pub struct Point { pub x:i64, pub y:i64 } pub fn point()->Point {}");
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
        &test_parameter_names(flat.function("point").unwrap()),
    )
    .unwrap()
    .to_string();
    flat.records.clear();
    let call = flat.function("point").unwrap();
    assert_eq!(
        test_return_packet(call, &(), &source_binding())
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        prebindgen_conversions::staged::emit_call(
            call,
            &(),
            &source_binding(),
            &syn::parse_quote!(_),
            &test_parameter_names(call)
        )
        .unwrap()
        .to_string(),
        before
    );
    assert_eq!(
        test_return_packet(call, &(), &source_binding())
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
            identity: prebindgen_conversions::ResourceIdentity::Single(quote!(#index)),
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
        TestAbi::wires(&plan.node).nth(1).unwrap().kind.component,
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
        TestAbi::wires(&plan.outcomes.success)
            .nth(1)
            .unwrap()
            .kind
            .component,
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
    assert!(TestAbi::wires(&plan.outcomes.success).len() == 0);
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
                outcomes: prebindgen_flat::Outcomes {
                    success: Default::default(),
                    source_error: None,
                    binding_failure: None,
                },
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
    let declarations = Declarations::new(&flat, vec![], vec![], vec![]).unwrap();
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
                outcomes: prebindgen_flat::Outcomes {
                    success: Default::default(),
                    source_error: None,
                    binding_failure: None,
                },
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
    fn check(layout: &prebindgen_flat::WireNode<TestWire, TestGroup, TestGroup>) {
        assert_eq!(
            prebindgen_flat::wire_definitions(std::iter::once(layout)).count(),
            4
        );
        assert_eq!(TestAbi::wires(layout).len(), 4);
        let WireNode::Branch {
            meta: metadata,
            children,
            ..
        } = layout
        else {
            panic!()
        };
        assert!(matches!(metadata, TestGroup::Record));
        let mut names = Vec::new();
        let mut widths = Vec::new();
        for node in children.as_slice() {
            let WireNode::Branch { meta: metadata, .. } = node else {
                panic!()
            };
            let TestGroup::Member(name) = metadata else {
                panic!()
            };
            names.push(name.to_string());
            widths.push(TestAbi::wires(node).len());
        }
        assert_eq!(names, ["maybe", "empty", "pair"]);
        assert_eq!(widths, [2, 0, 2]);
        for entry in prebindgen_flat::wire_definitions_with_paths(std::iter::once(layout)) {
            assert!(std::ptr::eq(
                entry.wire,
                prebindgen_flat::wire_at_path(std::iter::once(layout), &entry.path).unwrap()
            ));
        }
    }
    check(
        &flat.function("echo").unwrap().params[0]
            .ty
            .slot
            .as_ref()
            .unwrap()
            .node,
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
        check(&plan.outcomes.success);
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
        vec![],
        vec![],
        vec![(syn::parse_quote!(E), "message".into())],
    )
    .unwrap();
    defaults
        .fill(&mut flat, &TEST_CODEC, &fun!(echo), &Extra::default())
        .unwrap();
    let output = flat.function("echo").unwrap().ret.slot.as_ref().unwrap();
    assert_eq!(TestAbi::wires(&output.outcomes.success).len(), 2);
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
    for mapper in [None, Some("wrong"), Some("missing")] {
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
    assert!(Declarations::new(
        &flat,
        vec![],
        vec![expand_return!(Result<i64,String>).field_self().into()],
        vec![]
    )
    .is_err());
    assert!(Declarations::new(
        &flat,
        vec![expand_param!(Result<i64,String>).variant_self().into()],
        vec![],
        vec![]
    )
    .is_err());
}

#[test]
fn result_error_defaults_are_explicit_unique_and_selected_with_helpers() {
    let flat = parse("pub type E=String;pub fn message(e:E)->String{}pub fn f()->Result<i64,E>{}");
    assert!(Declarations::new(
        &flat,
        vec![],
        vec![],
        vec![
            (syn::parse_quote!(E), "message".into()),
            (syn::parse_quote!(E), "message".into())
        ]
    )
    .is_err());
    let errors = vec![(syn::parse_quote!(E), "message".into())];
    let names =
        declarations::source_functions(&[], &[], &errors, [&fun!(f).on_error(fun!(message))]);
    assert!(names.contains(&"f".into()) && names.contains(&"message".into()));
}

#[test]
fn selector_schema_separates_control_from_mutually_exclusive_payloads() {
    use prebindgen_flat::WireNode;
    let mut model = parse("pub struct Item {} pub fn empty()->Item{} pub fn numbered(n:i32)->Item{} pub fn take(v:Item){}");
    bind_input(
        &mut model,
        &TEST_CODEC,
        "take",
        "v",
        &Input::selector([
            Input::constructor("empty", []),
            Input::constructor("numbered", [Input::Leaf]),
        ]),
        &Extra::default(),
    )
    .unwrap();
    let input = model.function("take").unwrap().input_conversions().unwrap()[0];
    let WireNode::Branch {
        children: alternatives,
        controls,
        ..
    } = input.input_node()
    else {
        panic!()
    };
    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(&alternatives[0])).count(),
        0
    );
    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(&alternatives[1])).count(),
        1
    );
    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(input.input_node())).count(),
        2
    );
    assert_eq!(TestAbi::wires(&input.node).len(), 2);
    assert_eq!(controls.len(), 1);
    assert!(matches!(controls[0].kind.kind, Kind::Selector));
    assert_eq!(
        TestAbi::wires(&input.node).next().unwrap().name,
        prebindgen_flat::wire_definitions(std::iter::once(input.input_node()))
            .next()
            .unwrap()
            .name
    );
}

struct TestPacketValue<'a> {
    wire: &'a Wire<TestWire>,
}
fn test_return_packet<'a>(
    f: &'a prebindgen_flat::Function<InputPlan<TestWire>, OutputPlan<TestWire>>,
    _context: &(),
    _source: &syn::Ident,
) -> Result<std::vec::IntoIter<TestPacketValue<'a>>> {
    let plan = f.output_conversion()?;
    prebindgen_conversions::match_output_fields(
        TestAbi::wires(&plan.outcomes.success),
        &plan.values,
    )?
    .into_iter()
    .map(|(wire, _)| Ok(TestPacketValue { wire }))
    .collect::<Result<Vec<_>>>()
    .map(Vec::into_iter)
}
fn test_parameter_names(
    f: &prebindgen_flat::Function<InputPlan<TestWire>, OutputPlan<TestWire>>,
) -> Vec<syn::Ident> {
    f.input_conversions()
        .unwrap()
        .into_iter()
        .flat_map(|p| TestAbi::wires(&p.node))
        .map(|w| w.name.clone())
        .collect()
}

#[test]
fn nested_output_completion_channels_are_rejected_without_losing_unit_errors() {
    for source_error in [false, true] {
        let mut child = TEST_CODEC
            .output_leaf(
                &syn::parse_quote!(i32),
                &Extra::default(),
                syn::parse_quote!(out),
                quote!(value),
                false,
            )
            .unwrap();
        if source_error {
            child.outcomes.source_error = Some(WireNode::default());
        } else {
            child.outcomes.binding_failure = Some(WireNode::default());
        }
        assert!(optional_output(
            &TEST_CODEC,
            &Extra::default(),
            syn::parse_quote!(presence),
            syn::parse_quote!(payload),
            quote!(value),
            false,
            child.clone(),
        )
        .unwrap_err()
        .contains("outermost"));
        assert!(sequence::output(
            &TEST_CODEC,
            &syn::parse_quote!(Vec<i32>),
            &Extra::default(),
            syn::parse_quote!(rows),
            syn::parse_quote!(element),
            syn::parse_quote!(length),
            syn::parse_quote!(index),
            quote!(value),
            false,
            ProjectionMode::Owned,
            child.clone(),
            SequenceDelivery::Value,
        )
        .unwrap_err()
        .contains("outermost"));
        assert!(record_output(
            &TEST_CODEC,
            syn::parse_quote!(record),
            quote!(value),
            [syn::parse_quote!(field)],
            |_, _| Ok(child.clone()),
        )
        .unwrap_err()
        .contains("outermost"));
        let mut parent = child.clone();
        parent.outcomes = prebindgen_flat::Outcomes {
            success: WireNode::Branch {
                controls: vec![],
                children: vec![],
                meta: TestGroup::Values,
            },
            source_error: None,
            binding_failure: None,
        };
        assert!(append(&mut parent, child)
            .unwrap_err()
            .contains("outermost"));
        assert!(
            matches!(parent.outcomes.success, WireNode::Branch { children: elements, .. } if elements.is_empty())
        );
    }
}

#[test]
fn constructor_and_output_components_keep_groups_at_every_arity() {
    let mut flat = parse("pub fn zero()->i64{} pub fn one(a:i64)->i64{} pub fn two(a:i64,b:i64)->i64{} pub fn observe(v:&i64)->i64{} pub fn f(value:i64)->i64{}");
    for (name, count) in [("zero", 0), ("one", 1), ("two", 2)] {
        bind_input(
            &mut flat,
            &TEST_CODEC,
            "f",
            "value",
            &Input::constructor(name, vec![Input::Leaf; count]),
            &Extra::default(),
        )
        .unwrap();
        let input = flat.function("f").unwrap().params[0]
            .ty
            .slot
            .as_ref()
            .unwrap();
        assert!(
            matches!(&input.node, WireNode::Branch { meta: TestGroup::Values, children: elements , .. } if elements.len() == count)
        );
        if count == 0 {
            continue;
        } // Empty combinations are intentionally rejected.
        bind_output(
            &mut flat,
            &TEST_CODEC,
            "f",
            &Output::combine(vec![Output::getter("observe", Output::Leaf); count]),
            &Extra::default(),
        )
        .unwrap();
        let output = flat.function("f").unwrap().ret.slot.as_ref().unwrap();
        assert!(
            matches!(&output.outcomes.success, WireNode::Branch { meta: TestGroup::Values, children: elements , .. } if elements.len() == count)
        );
    }
}

#[test]
fn malformed_error_channel_is_rejected_before_replacing_output() {
    let mut flat = parse("pub fn f()->i64{}");
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "f",
        &Output::Leaf,
        &Extra::default(),
    )
    .unwrap();
    let before = flat
        .function("f")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap()
        .statements
        .to_string();
    let error = bind_output(
        &mut flat,
        &TEST_CODEC,
        "f",
        &Output::Leaf,
        &Extra {
            name: "malformed_binding_failure",
            ..Extra::default()
        },
    )
    .unwrap_err();
    assert!(error.contains("no alternatives"), "{error}");
    let unchanged = flat.function("f").unwrap().ret.slot.as_ref().unwrap();
    assert!(unchanged.outcomes.binding_failure.is_none());
    assert_eq!(unchanged.statements.to_string(), before);
}

#[test]
fn failed_metadata_construction_preserves_installed_conversions() {
    let mut flat=parse("pub fn empty()->Option<i64>{} pub fn some(v:i64)->Option<i64>{} pub fn f(v:Option<i64>)->Option<i64>{}");
    let optional = Input::Optional(Box::new(Input::Leaf));
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "f",
        "v",
        &optional,
        &Extra::default(),
    )
    .unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "f",
        &Output::Optional(Box::new(Output::Leaf)),
        &Extra::default(),
    )
    .unwrap();
    let input_before = flat.function("f").unwrap().params[0]
        .ty
        .slot
        .as_ref()
        .unwrap()
        .validation
        .to_string();
    let output_before = flat
        .function("f")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap()
        .statements
        .to_string();
    for (recipe, name, expected) in [
        (
            optional,
            "reject_optional_metadata",
            "optional metadata rejected",
        ),
        (
            Input::selector([
                Input::constructor("empty", []),
                Input::constructor("some", [Input::Leaf]),
            ]),
            "reject_alternative_metadata",
            "alternative metadata rejected",
        ),
    ] {
        let error = bind_input(
            &mut flat,
            &TEST_CODEC,
            "f",
            "v",
            &recipe,
            &Extra {
                name,
                ..Extra::default()
            },
        )
        .unwrap_err();
        assert_eq!(error, expected);
        assert_eq!(
            flat.function("f").unwrap().params[0]
                .ty
                .slot
                .as_ref()
                .unwrap()
                .validation
                .to_string(),
            input_before
        );
    }
    let error = bind_output(
        &mut flat,
        &TEST_CODEC,
        "f",
        &Output::Optional(Box::new(Output::Leaf)),
        &Extra {
            name: "reject_optional_metadata",
            ..Extra::default()
        },
    )
    .unwrap_err();
    assert_eq!(error, "optional metadata rejected");
    assert_eq!(
        flat.function("f")
            .unwrap()
            .ret
            .slot
            .as_ref()
            .unwrap()
            .statements
            .to_string(),
        output_before
    );
}

#[test]
fn resolved_flat_is_the_only_source_model_needed_by_compilation() {
    let source = parse("pub struct R { pub n:i64 } pub fn make(n:i64)->R{} pub fn inspect(r:&R)->i64{} pub fn echo(r:Option<R>)->Option<R>{}")
        .map_slots(|_, _| Ok(None::<()>), |_, _| Ok(None::<()>)).unwrap();
    let defaults = Declarations::new(
        &source,
        vec![expand_param!(R).variant(fun!(make)).into()],
        vec![expand_return!(R).field(fun!(inspect)).into()],
        vec![],
    )
    .unwrap();
    let resolved = defaults
        .resolve(source, &TEST_CODEC, &[fun!(echo)], &Extra::default())
        .unwrap();
    drop(defaults);
    let function = resolved.function("echo").unwrap();
    let Input::Optional(child) = function.params[0].ty.slot.as_ref().unwrap() else {
        panic!("expected resolved Option");
    };
    assert!(
        matches!(child.as_ref(), Input::Constructor { function, args } if function == "make" && matches!(args.as_slice(), [Input::Leaf]))
    );
    let Output::Optional(child) = function.ret.slot.as_ref().unwrap() else {
        panic!("expected resolved Option");
    };
    assert!(matches!(child.as_ref(), Output::Getter { function, .. } if function == "inspect"));
    let compiled = compile(resolved, &TEST_CODEC, &Extra::default()).unwrap();
    assert!(compiled.function("echo").unwrap().params[0]
        .ty
        .slot
        .is_some());
    assert!(compiled.function("echo").unwrap().ret.slot.is_some());
    assert!(compiled.function("make").unwrap().ret.slot.is_none());
}

#[test]
fn callback_variant_selects_delivery_before_wires_exist() {
    let mut flat = parse("pub fn f()->Option<Vec<i64>>{}");
    let defaults = Declarations::new(&flat, vec![], vec![], vec![]).unwrap();
    let variant = defaults
        .callback_variant(
            &mut flat,
            &TEST_CODEC,
            &fun!(f),
            "f_fold",
            &Extra::default(),
        )
        .unwrap();
    assert!(flat.function("f_fold").unwrap().ret.slot.is_none());
    let Some(Output::Optional(child)) = variant.ret else {
        panic!("missing callback description")
    };
    assert!(matches!(
        *child,
        Output::Sequence {
            delivery: SequenceDelivery::Callback,
            ..
        }
    ));
}

#[test]
fn sequence_identity_uses_element_defaults_and_staged_errors_name_the_function() {
    let source = model()
        .map_slots(|_, _| Ok(None::<()>), |_, _| Ok(None::<()>))
        .unwrap();
    let defaults = Declarations::new(
        &source,
        vec![
            expand_param!(Payload).variant(fun!(make)).into(),
            expand_param!(Vec<Payload>).variant_self().into(),
        ],
        vec![],
        vec![],
    )
    .unwrap();
    let resolved = defaults
        .resolve_input(
            &source,
            &TEST_CODEC,
            &source
                .resolve_type(&syn::parse_quote!(Vec<Payload>))
                .unwrap(),
            &Input::auto(),
            &Extra::default(),
        )
        .unwrap();
    assert!(
        matches!(resolved,Input::Sequence(child) if matches!(&*child,Input::Constructor {function,..} if function=="make"))
    );
    let error = defaults
        .resolve(
            source,
            &TEST_CODEC,
            &[fun!(foo).input("x", Input::call("missing"))],
            &Extra::default(),
        )
        .unwrap_err();
    assert!(error.starts_with("foo:"), "{error}");

    let invalid = parse("pub fn broken()->i64{}")
        .map_slots(
            |_, _| Ok(None::<Input>),
            |_, _| Ok(Some(Output::Optional(Box::new(Output::Leaf)))),
        )
        .unwrap();
    let error = compile(invalid, &TEST_CODEC, &Extra::default()).unwrap_err();
    assert!(error.starts_with("broken:"), "{error}");
}

#[test]
fn omitted_typed_fields_drop_all_enclosing_option_and_sequence_wires() {
    let mut model = parse("pub type Hidden=String;pub type Other=u32;pub struct Record{pub optional:Option<Hidden>,pub nested:Option<Vec<Option<Hidden>>>,pub visible:i64}pub fn parts(value:Record)->Record{}pub fn read()->Record{}");
    let output: Output<OutputRequest> = expand_return!(Record)
        .fields_into(
            fields!(parts)
                .field("optional", expand_return!(Hidden))
                .field_override("nested", expand_return!(Hidden)),
        )
        .into();
    bind_output(&mut model, &TEST_CODEC, "read", &output, &Extra::default()).unwrap();
    let plan = model.function("read").unwrap().ret.slot.as_ref().unwrap();
    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(&plan.outcomes.success)).count(),
        1
    );
    assert_eq!(plan.values.len(), 1);
    let invalid: Output<OutputRequest> = expand_return!(Record)
        .fields_into(fields!(parts).field("optional", expand_return!(Other)))
        .into();
    assert!(bind_output(&mut model, &TEST_CODEC, "read", &invalid, &Extra::default()).is_err());
}

#[test]
fn named_getter_resolution_preserves_the_selected_name_in_the_tree() {
    let model = parse("pub type T=String;pub fn length(value:&T)->i64{}");
    let defaults = Declarations::new(&model, vec![], vec![], vec![]).unwrap();
    let tree: Output<OutputRequest> = expand_return!(T).field(fun!(length).name("size")).into();
    let tree = defaults
        .resolve_output(
            &model,
            &TEST_CODEC,
            &model.resolve_type(&syn::parse_quote!(T)).unwrap(),
            &tree,
            &Extra::default(),
        )
        .unwrap();
    assert!(
        matches!(tree,Output::Named {name,output} if name=="size" && matches!(output.as_ref(),Output::Getter {function,output} if function=="length"&&matches!(output.as_ref(),Output::Leaf)))
    );
}

#[test]
fn helper_references_reject_binding_only_decorations() {
    assert!(
        std::panic::catch_unwind(|| expand_param!(T).variant(fun!(make).name("renamed"))).is_err()
    );
    assert!(std::panic::catch_unwind(|| fun!(f).on_error(fun!(message).name("renamed"))).is_err());
    assert!(std::panic::catch_unwind(
        || expand_return!(T).field(fun!(get).expand_return(expand_return!(i64).field_self()))
    )
    .is_err());
}

#[test]
fn nested_requests_resolve_into_strict_schemas_before_compilation() {
    let source = parse("pub type R=String;pub fn make(a:i32,b:Option<i64>)->R{}pub fn narrow(v:i16)->i64{}pub fn run(value:R)->i64{}")
        .map_slots(|_,_|Ok(None::<()>),|_,_|Ok(None::<()>)).unwrap();
    let defaults = Declarations::new(
        &source,
        vec![expand_param!(i64).variant(fun!(narrow)).into()],
        vec![],
        vec![],
    )
    .unwrap();
    let requested: Input<InputRequest> = Input::constructor(
        "make",
        [Input::Leaf, Input::Optional(Box::new(Input::auto()))],
    );
    let resolved: Flat<Input, Output> = defaults
        .resolve(
            source,
            &TEST_CODEC,
            &[fun!(run).input("value", requested)],
            &Extra::default(),
        )
        .unwrap();
    let Input::Constructor { args, .. } = resolved.function("run").unwrap().params[0]
        .ty
        .slot
        .as_ref()
        .unwrap()
    else {
        panic!("constructor not resolved")
    };
    assert!(matches!(&args[0], Input::Leaf));
    assert!(
        matches!(&args[1],Input::Optional(child) if matches!(child.as_ref(),Input::Constructor {function,args} if function=="narrow"&&matches!(args.as_slice(),[Input::Leaf])))
    );
    compile(resolved, &TEST_CODEC, &Extra::default()).unwrap();
}

#[test]
fn completed_subtrees_keep_explicit_choices_when_used_in_preparation() {
    let source=parse("pub type R=String;pub fn direct(v:i64)->R{}pub fn narrow(v:i16)->i64{}pub fn observe(r:&R)->i64{}pub fn sign(v:&i64)->i8{}pub fn run(value:R,count:i64)->R{}")
        .map_slots(|_,_|Ok(None::<()>),|_,_|Ok(None::<()>)).unwrap();
    let defaults = Declarations::new(
        &source,
        vec![expand_param!(i64).variant(fun!(narrow)).into()],
        vec![expand_return!(i64).field(fun!(sign)).into()],
        vec![],
    )
    .unwrap();
    let input: Input = Input::constructor("direct", [Input::Leaf]);
    let output: Output = Output::getter("observe", Output::Leaf);
    let decl = fun!(run)
        .input("value", input.into_preparation())
        .output(output.into_preparation());
    let resolved = defaults
        .resolve(source, &TEST_CODEC, &[decl], &Extra::default())
        .unwrap();
    let run = resolved.function("run").unwrap();
    assert!(
        matches!(run.params[0].ty.slot.as_ref().unwrap(),Input::Constructor{function,args} if function=="direct"&&matches!(args.as_slice(),[Input::Leaf]))
    );
    assert!(
        matches!(run.params[1].ty.slot.as_ref().unwrap(),Input::Constructor{function,..} if function=="narrow")
    );
    assert!(
        matches!(run.ret.slot.as_ref().unwrap(),Output::Getter{function,output} if function=="observe"&&matches!(output.as_ref(),Output::Leaf))
    );
    compile(resolved, &TEST_CODEC, &Extra::default()).unwrap();
}

#[test]
fn result_resolution_consumes_nested_requests_and_preserves_error_mapper() {
    let source = parse("pub type E=String;pub fn message(e:&E)->String{}pub fn sign(v:&i64)->i8{}pub fn run()->Result<Option<i64>,E>{}")
        .map_slots(|_,_|Ok(None::<()>),|_,_|Ok(None::<()>)).unwrap();
    let defaults = Declarations::new(
        &source,
        vec![],
        vec![expand_return!(i64).field(fun!(sign)).into()],
        vec![],
    )
    .unwrap();
    let resolved = defaults
        .resolve(
            source,
            &TEST_CODEC,
            &[fun!(run)
                .output(Output::Optional(Box::new(Output::auto())))
                .on_error(fun!(message))],
            &Extra::default(),
        )
        .unwrap();
    let Output::Result { ok, error } = resolved.function("run").unwrap().ret.slot.as_ref().unwrap()
    else {
        panic!("missing Result")
    };
    assert_eq!(error.as_deref(), Some("message"));
    assert!(
        matches!(ok.as_ref(),Output::Optional(child) if matches!(child.as_ref(),Output::Getter {function,output} if function=="sign" && matches!(output.as_ref(),Output::Leaf)))
    );
    let compiled = compile(resolved, &TEST_CODEC, &Extra::default()).unwrap();
    assert!(compiled
        .function("run")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap()
        .statements
        .to_string()
        .contains("message (&"));
}

#[test]
fn resolved_omissions_and_sequence_policies_survive_preparation_round_trip() {
    let source = parse("pub type Hidden=String;pub struct R{pub hidden:Option<Vec<Option<Hidden>>>,pub rows:Vec<i64>}pub fn read()->R{}");
    let defaults = Declarations::new(&source, vec![], vec![], vec![]).unwrap();
    let tree = Output::OwnedFields(vec![
        Output::for_type(syn::parse_quote!(Hidden), Output::Discard),
        Output::Sequence {
            delivery: SequenceDelivery::Callback,
            element: Box::new(Output::auto()),
            mode: Some(ProjectionMode::Owned),
        },
    ]);
    let resolved = defaults
        .resolve_output(
            &source,
            &TEST_CODEC,
            &source.function("read").unwrap().ret.ty,
            &tree,
            &Extra::default(),
        )
        .unwrap();
    assert!(
        matches!(&resolved,Output::OwnedFields(fields) if matches!(fields.as_slice(),[Output::Discard,Output::Sequence{delivery:SequenceDelivery::Callback,mode:Some(ProjectionMode::Owned),..}]))
    );
    let round_trip = defaults
        .resolve_output(
            &source,
            &TEST_CODEC,
            &source.function("read").unwrap().ret.ty,
            &resolved.clone().into_preparation(),
            &Extra::default(),
        )
        .unwrap();
    assert_eq!(format!("{resolved:?}"), format!("{round_trip:?}"));
}
