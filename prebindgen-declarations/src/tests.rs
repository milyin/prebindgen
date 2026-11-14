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
type TestBindings<W> = super::PlanBindings<W, (), TestGroup, TestGroup>;
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
    test_operations!();
    type Schema = prebindgen_conversions::PlanSchema<TestWire, (), TestGroup, TestGroup>;
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
    fn mapped_error(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        _: &Type,
        message: TokenStream,
        _: &Extra,
    ) -> Result<TokenStream> {
        Ok(message)
    }

    type Chain = TestGroup;
    type Branch = TestGroup;

    fn validate_input(&self, plan: &CodecInputPlan<Self>) -> Result<()> {
        TestAbi::validate(&plan.node)
    }
    fn validate_output(&self, plan: &CodecOutputPlan<Self>) -> Result<()> {
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

    type Extra = Extra;
    type Wire = TestWire;
    type Context = ();

    fn selector(&self, extra: &Extra) -> Result<TestWire> {
        Ok({
            TestWire {
                kind: Kind::Selector,
                name: extra.name,
                ty: "i32".into(),
                component: Default::default(),
            }
        })
    }
    fn output_consumes(
        &self,
        _flat: &prebindgen_flat::Flat<Self::Schema>,
        ty: &Type,
        extra: &Extra,
    ) -> bool {
        Value::of(ty).is_none() && !extra.copy
    }
    fn input_leaf(&self, request: InputLeaf<'_, Self>) -> Result<InputPlan<TestWire>> {
        let InputLeaf {
            flat: _flat,
            ty,
            extra,
            wire: w,
            target: v,
            ..
        } = request;

        let name = extra.name;
        if let Some(value) = Value::of(ty) {
            let decode = quote!(decode(#name, #w)?);
            return value::input(ty, w, v, decode, wire(Kind::Value(value), ty, extra));
        }
        Ok({
            let validation = quote!();
            let acquire = quote!();
            let statements = quote!(let #v=receive_token(#w););
            let mut plan = InputPlan::new(
                prebindgen_flat::WireNode::Leaf(vec![Wire {
                    name: w.clone(),
                    kind: wire(Kind::Token, ty, extra),
                }]),
                quote!(#validation #acquire #statements),
                quote!(#v),
            );
            plan.fallible = false;
            plan
        })
    }
    fn output_leaf(&self, request: OutputLeaf<'_, Self>) -> Result<OutputPlan<TestWire>> {
        let OutputLeaf {
            flat: _flat,
            ty,
            extra,
            wire: w,
            value: v,
            access,
            ..
        } = request;
        let borrowed = access.is_shared();

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

    fn group(&self, kind: GroupKind<'_>) -> Result<TestGroup> {
        Ok(match kind {
            GroupKind::Values | GroupKind::NamedOutput(_) => TestGroup::Values,
            GroupKind::Fields => TestGroup::Record,
            GroupKind::Field(name) => TestGroup::Member(name.clone()),
        })
    }
    fn optional(&self, request: Optional<'_, Self>) -> Result<OptionalEncoding<Self>> {
        let prepared = prepare_optional_payload(
            TestAbi::wires_mut(request.payload),
            request.direction,
            |w| self.niche(w),
            |w, n| {
                self.set_niche(w, n);
                Ok(())
            },
            |w, n, v| self.is_present(w, n, v),
            |w, n| self.remaining_niche(w, n),
            |w, n| self.absent_value(w, n, false),
        )?;
        if request.extra.name == "reject_optional_metadata" {
            return Err("optional metadata rejected".into());
        }
        let metadata = TestGroup::Maybe;
        let presence = if prepared.reservation.is_some() {
            Presence::Niche {
                input_test: prepared.input_test,
            }
        } else {
            Presence::Flag(Wire {
                name: request.presence,
                kind: self.selector(request.extra)?,
            })
        };
        Ok(OptionalEncoding {
            presence,
            metadata,
            absent_values: prepared.absent_values,
        })
    }
    fn alternatives(&self, request: Alternatives<'_, Self>) -> Result<TestGroup> {
        for arm in request.arms {
            retire_optional_niches(
                TestAbi::wires_mut(arm),
                None,
                |w, n| self.remaining_niche(w, n),
                |w, n| {
                    self.set_niche(w, n);
                    Ok(())
                },
            )?;
        }
        if request.selector.kind.name == "reject_alternative_metadata" {
            return Err("alternative metadata rejected".into());
        }
        Ok(TestGroup::Maybe)
    }
}

impl TestCodec {
    fn niche(&self, _: &TestWire) -> Option<()> {
        None
    }
    fn set_niche(&self, _: &mut TestWire, _: Option<()>) {}
    fn absent_value(&self, _: &TestWire, _: Option<&()>, _: bool) -> Result<TokenStream> {
        Ok(quote!(Default::default()))
    }
    fn is_present(&self, _: &TestWire, _: &(), _: TokenStream) -> Result<TokenStream> {
        unreachable!()
    }
    fn remaining_niche(&self, _: &TestWire, _: &()) -> Option<()> {
        None
    }
}

fn parse(source: &str) -> TestBindings<TestWire> {
    Flat::parse(
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap()
}
fn model() -> TestBindings<TestWire> {
    parse("pub struct Payload { pub data: Vec<u8> } pub fn make(s: String) -> Payload {} pub fn foo(x: Payload, y: Payload) -> Payload {}")
}
fn extra(name: &'static str) -> Extra {
    Extra { name, copy: false }
}
fn fill(flat: &mut TestBindings<TestWire>, decl: &TestFunction) -> Result<()> {
    TestDefaults::new(flat, vec![], vec![], vec![])?.fill(
        flat,
        &TEST_CODEC,
        decl,
        &Extra::default(),
    )
}
#[test]
fn different_occurrences_have_different_converters() {
    let mut flat = model();
    test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &TestInputConfig::selector([
            TestInputConfig::Leaf(TestInputLeafOp::Value),
            TestInputConfig::constructor("make", [TestInputConfig::Leaf(TestInputLeafOp::Value)]),
        ]),
        &extra("x"),
    )
    .unwrap();
    test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "y",
        &TestInputConfig::Leaf(TestInputLeafOp::Value),
        &extra("y"),
    )
    .unwrap();
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &TestOutputConfig::Leaf(TestOutputLeafOp::Value),
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
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &TestOutputConfig::Leaf(TestOutputLeafOp::Value),
        &Extra::default(),
    )
    .unwrap();
    assert!(validate_bindings(&flat).unwrap_err().contains("no input"));
}
#[test]
fn rejects_inputs_without_a_result_converter() {
    let mut flat = model();
    test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &TestInputConfig::Leaf(TestInputLeafOp::Value),
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
    fill(&mut flat, &test_fun!(foo)).unwrap();
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
    fill(&mut flat, &test_fun!(foo)).unwrap();
    let before = format!("{flat:?}");
    let error = test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &TestInputConfig::constructor("foo", []),
        &Extra::default(),
    )
    .unwrap_err();
    assert!(error.contains("argument count"));
    assert_eq!(format!("{flat:?}"), before);
    for output in [
        TestOutputConfig::combine([
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
        ]),
        TestOutputConfig::fields([
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
        ]),
    ] {
        assert!(
            test_bind_output(&mut flat, &TEST_CODEC, "foo", &output, &Extra::default()).is_err()
        );
        assert_eq!(format!("{flat:?}"), before);
    }
    assert!(test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "missing",
        &TestInputConfig::Leaf(TestInputLeafOp::Value),
        &Extra::default()
    )
    .unwrap_err()
    .contains("unknown parameter"));
}
#[test]
fn observations_precede_move_but_wires_keep_order() {
    let mut flat = model();
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &TestOutputConfig::combine([
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
            TestOutputConfig::fields([TestOutputConfig::Leaf(TestOutputLeafOp::Value)]),
        ]),
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
            test_expand_return!(Vec<u8>).field_self(),
            "unknown output field",
        ),
        (
            "data",
            test_expand_return!(String).field_self(),
            "does not match occurrence",
        ),
    ] {
        let mut flat = parse(source);
        let function = test_fun!(read).expand_return(
            test_expand_return!(Payload).fields(test_fields!(parts).field_override(field, output)),
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
    test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &TestInputConfig::selector([
            TestInputConfig::constructor("make", [TestInputConfig::Leaf(TestInputLeafOp::Value)]),
            TestInputConfig::Leaf(TestInputLeafOp::Value),
        ]),
        &extra("x"),
    )
    .unwrap();
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &TestOutputConfig::Leaf(TestOutputLeafOp::Value),
        &extra("result"),
    )
    .unwrap();
    let declarations = TestDefaults::new(
        &flat,
        vec![test_expand_param!(Payload)
            .variant(test_fun!(make))
            .variant_self()
            .into()],
        vec![],
        vec![],
    )
    .unwrap();
    declarations
        .fill(&mut flat, &TEST_CODEC, &test_fun!(foo), &extra("defaults"))
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
            &test_fun!(foo).expand_param("y", test_expand_param!(Payload).variant_self()),
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
    assert!(x.code.to_string().contains("\"x\""));
}
#[test]
fn declaration_compilation_is_atomic_even_when_output_fails() {
    let mut flat = model();
    fill(&mut flat, &test_fun!(foo)).unwrap();
    let before = format!("{flat:?}");
    let function = test_fun!(foo)
        .expand_param("x", test_expand_param!(Payload).variant(test_fun!(make)))
        .expand_return(test_expand_return!(Payload).field(test_fun!(make)));
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
            test_bind_input(
                model,
                &TEST_CODEC,
                "foo",
                parameter,
                &TestInputConfig::Leaf(TestInputLeafOp::Value),
                &Extra::default(),
            )
            .unwrap();
        }
        test_bind_output(
            model,
            &TEST_CODEC,
            "foo",
            &TestOutputConfig::Leaf(TestOutputLeafOp::Value),
            &Extra::default(),
        )
        .unwrap();
    }
    assert_eq!(format!("{flat:?}"), format!("{reversed:?}"));
    let old_y = format!("{:?}", flat.function("foo").unwrap().params[1].ty.slot);
    test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &TestInputConfig::selector([
            TestInputConfig::Leaf(TestInputLeafOp::Value),
            TestInputConfig::constructor("make", [TestInputConfig::Leaf(TestInputLeafOp::Value)]),
        ]),
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
    TestDefaults::new(
        &flat,
        vec![],
        vec![test_expand_return!(Key).field(test_fun!(key_text)).into()],
        vec![],
    )
    .unwrap()
    .fill(
        &mut flat,
        &TEST_CODEC,
        &test_fun!(read).expand_return(test_expand_return!(Payload).fields(test_fields!(parts))),
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
    let defaults = TestDefaults::new(
        &flat,
        vec![test_expand_param!(i64).variant_self().into()],
        vec![test_expand_return!(i64).field_self().into()],
        vec![],
    )
    .unwrap();
    defaults
        .fill(&mut flat, &TEST_CODEC, &test_fun!(f), &extra("scalar"))
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
    assert!(input.code.to_string().contains("scalar"));
    assert!(output.statements.to_string().contains("scalar"));
}
#[test]
fn nested_recipes_use_only_the_callers_extra() {
    let mut flat = parse("pub type Key=String; pub struct Payload { pub key:Key } pub fn make(k:Key)->Payload {} pub fn parts(p:&Payload)->Payload {} pub fn foo(x:Payload,y:Payload)->Payload {}");
    // Independently bound helpers must not affect the enclosing conversion.
    test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "make",
        "k",
        &TestInputConfig::Leaf(TestInputLeafOp::Value),
        &extra("constructor declaration"),
    )
    .unwrap();
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "parts",
        &TestOutputConfig::Leaf(TestOutputLeafOp::Value),
        &extra("getter declaration"),
    )
    .unwrap();
    test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &TestInputConfig::selector([
            TestInputConfig::Leaf(TestInputLeafOp::Value),
            TestInputConfig::constructor("make", [TestInputConfig::Leaf(TestInputLeafOp::Value)]),
        ]),
        &extra("x"),
    )
    .unwrap();
    test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "y",
        &TestInputConfig::fields([TestInputConfig::Leaf(TestInputLeafOp::Value)]),
        &extra("y"),
    )
    .unwrap();
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &TestOutputConfig::combine([
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
            TestOutputConfig::getter(
                "parts",
                TestOutputConfig::fields([TestOutputConfig::Leaf(TestOutputLeafOp::Value)]),
            ),
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
    let recipe = TestOutputConfig::combine([
        TestOutputConfig::Leaf(TestOutputLeafOp::Value),
        TestOutputConfig::Leaf(TestOutputLeafOp::Value),
    ]);
    for copy in [false, true] {
        let mut flat = model();
        let result = test_bind_output(
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
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "point",
        &TestOutputConfig::fields([
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
        ]),
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
        test_operations!();
        type Schema = prebindgen_conversions::PlanSchema<TestWire, (), TestGroup, TestGroup>;

        type Chain = TestGroup;
        type Branch = TestGroup;

        fn validate_input(&self, plan: &CodecInputPlan<Self>) -> Result<()> {
            TestAbi::validate(&plan.node)
        }
        fn validate_output(&self, plan: &CodecOutputPlan<Self>) -> Result<()> {
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

        type Extra = Extra;
        type Wire = TestWire;
        type Context = ();

        fn selector(&self, extra: &Extra) -> Result<TestWire> {
            TEST_CODEC.selector(extra)
        }
        fn output_consumes(
            &self,
            flat: &prebindgen_flat::Flat<Self::Schema>,
            ty: &Type,
            extra: &Extra,
        ) -> bool {
            TEST_CODEC.output_consumes(flat, ty, extra)
        }
        fn input_leaf(&self, request: InputLeaf<'_, Self>) -> Result<InputPlan<TestWire>> {
            let InputLeaf {
                flat,
                ty,
                extra,
                wire: w,
                target: v,
                active,
                ..
            } = request;

            let plan = TEST_CODEC.input_leaf(crate::InputLeaf::new(
                flat,
                ty,
                extra,
                w.clone(),
                v,
                active,
            ))?;
            Ok(plan)
        }
        fn output_leaf(&self, request: OutputLeaf<'_, Self>) -> Result<OutputPlan<TestWire>> {
            let OutputLeaf {
                flat,
                ty,
                extra,
                wire: w,
                value: v,
                access,
                ..
            } = request;
            let borrowed = access.is_shared();

            TEST_CODEC.output_leaf(crate::OutputLeaf::new(
                flat,
                ty,
                extra,
                w,
                v,
                if borrowed {
                    crate::ValueAccess::Shared
                } else {
                    crate::ValueAccess::Owned
                },
            ))
        }
        fn finish_input(&self, plan: &mut InputPlan<TestWire>, extra: &Extra) -> Result<()> {
            if extra.name == "conflict" {
                return Err("input finalization failed".into());
            }
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
        fn group(&self, kind: GroupKind<'_>) -> Result<TestGroup> {
            TEST_CODEC.group(kind)
        }
        fn optional(&self, request: Optional<'_, Self>) -> Result<OptionalEncoding<Self>> {
            let e = TEST_CODEC.optional(Optional {
                extra: request.extra,
                payload: request.payload,
                presence: request.presence,
                direction: request.direction,
            })?;
            Ok(OptionalEncoding {
                presence: e.presence,
                metadata: e.metadata,
                absent_values: e.absent_values,
            })
        }
        fn alternatives(&self, request: Alternatives<'_, Self>) -> Result<TestGroup> {
            TEST_CODEC.alternatives(Alternatives {
                selector: request.selector,
                arms: request.arms,
            })
        }
    }
    let source = "pub struct Pair { pub a:i32, pub b:i32 } pub fn pair(a:i32,b:i32)->Pair {} pub fn take(p:Pair) {}";
    let mut conflicting = parse(source);
    assert!(test_bind_input(
        &mut conflicting,
        &FinalizingCodec,
        "take",
        "p",
        &TestInputConfig::constructor(
            "pair",
            [
                TestInputConfig::Leaf(TestInputLeafOp::Value),
                TestInputConfig::Leaf(TestInputLeafOp::Value)
            ]
        ),
        &Extra {
            name: "conflict",
            ..Extra::default()
        }
    )
    .unwrap_err()
    .contains("input finalization failed"));
    assert!(conflicting.function("take").unwrap().params[0]
        .ty
        .slot
        .is_none());
    let mut direct = parse(source);
    test_bind_input(
        &mut direct,
        &FinalizingCodec,
        "take",
        "p",
        &TestInputConfig::constructor(
            "pair",
            [
                TestInputConfig::Leaf(TestInputLeafOp::Value),
                TestInputConfig::Leaf(TestInputLeafOp::Value),
            ],
        ),
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
    test_bind_output(
        &mut direct,
        &FinalizingCodec,
        "take",
        &TestOutputConfig::fields([]),
        &Extra::default(),
    )
    .unwrap();
    let declarations = TestDefaults::new(&direct, vec![], vec![], vec![]).unwrap();
    let decl = TestFunction::new(syn::parse_quote!(take)).input(
        "p",
        TestInputConfig::for_type(
            syn::parse_quote!(Pair),
            TestInputConfig::Branch {
                operation: TestInputBranch::Selector,
                children: vec![],
            },
        )
        .variant("pair"),
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
        "take: finalization failed"
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
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "point",
        &TestOutputConfig::fields([
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
            TestOutputConfig::Leaf(TestOutputLeafOp::Value),
        ]),
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
fn shared_record_input_composes_complete_fields_and_hoists_backing_storage() {
    let make = |index: usize| {
        let target = format_ident!("field{index}");
        let mut child = TEST_CODEC
            .input_leaf(crate::InputLeaf::new(
                &parse(""),
                &syn::parse_quote!(i64),
                &Extra::default(),
                format_ident!("wire{index}"),
                target.clone(),
                &Condition::default(),
            ))
            .unwrap();
        child.storage = quote!(let #target=(););
        Ok((target, child))
    };
    let plan = record_input(
        &TEST_CODEC,
        &syn::parse_quote!(Pair),
        format_ident!("pair"),
        [make(0), make(1)],
    )
    .unwrap();
    assert!(plan.storage.to_string().contains("field0"));
    assert!(plan.storage.to_string().contains("field1"));
    let code = plan.code.to_string();
    assert!(code.find("wire0").unwrap() < code.find("wire1").unwrap());
    assert_eq!(
        TestAbi::wires(&plan.node).nth(1).unwrap().kind.component,
        ComponentPath(vec![ComponentSegment::Field("field1".into())])
    );
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
                let mut child = TEST_CODEC.output_leaf(crate::OutputLeaf::new(
                    &parse(""),
                    &syn::parse_quote!(i64),
                    &Extra::default(),
                    format_ident!("wire{i}"),
                    field,
                    crate::ValueAccess::Shared,
                ))?;
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
    let declarations = TestDefaults::new(&flat, vec![], vec![], vec![]).unwrap();
    declarations
        .fill(
            &mut flat,
            &TEST_CODEC,
            &test_fun!(make).expand_return(
                test_expand_return!(Pair)
                    .fields_into(
                        test_fields!(into_pair)
                            .field("first", test_expand_return!(i64).field_self_ref()),
                    )
                    .field(test_fun!(count)),
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
            &test_fun!(make)
                .expand_return(test_expand_return!(Pair).fields_into(test_fields!(copy_pair))),
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
            &test_fun!(make)
                .expand_return(test_expand_return!(Pair).fields(test_fields!(into_pair))),
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
    let input = TestInputConfig::Branch {
        operation: TestInputBranch::Fields,
        children: vec![
            TestInputConfig::Unary {
                operation: TestInputUnary::Optional,
                child: Box::new(TestInputConfig::Leaf(TestInputLeafOp::Value)),
            },
            TestInputConfig::Branch {
                operation: TestInputBranch::Fields,
                children: vec![],
            },
            TestInputConfig::Branch {
                operation: TestInputBranch::Fields,
                children: vec![
                    TestInputConfig::Leaf(TestInputLeafOp::Value),
                    TestInputConfig::Leaf(TestInputLeafOp::Value),
                ],
            },
        ],
    };
    test_bind_input(
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
            TestOutputConfig::Unary {
                operation: TestOutputUnary::Optional,
                child: Box::new(TestOutputConfig::Leaf(TestOutputLeafOp::Value)),
            },
            TestOutputConfig::Branch {
                operation: TestOutputBranch::Fields,
                children: vec![],
            },
            TestOutputConfig::Branch {
                operation: TestOutputBranch::Fields,
                children: vec![
                    TestOutputConfig::Leaf(TestOutputLeafOp::Value),
                    TestOutputConfig::Leaf(TestOutputLeafOp::Value),
                ],
            },
        ];
        let output = if owned {
            TestOutputConfig::Branch {
                operation: TestOutputBranch::OwnedFields,
                children: fields,
            }
        } else {
            TestOutputConfig::Branch {
                operation: TestOutputBranch::Fields,
                children: fields,
            }
        };
        test_bind_output(&mut flat, &TEST_CODEC, "echo", &output, &Extra::default()).unwrap();
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
            .input_leaf(crate::InputLeaf::new(
                &parse(""),
                &syn::parse_quote!(i64),
                &Extra::default(),
                format_ident!("w{i}"),
                format_ident!("v{i}"),
                &Condition::default(),
            ))
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
            TEST_CODEC.output_leaf(crate::OutputLeaf::new(
                &parse(""),
                &syn::parse_quote!(i64),
                &Extra::default(),
                format_ident!("w{i}"),
                value,
                if !owned {
                    crate::ValueAccess::Shared
                } else {
                    crate::ValueAccess::Owned
                },
            ))
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
    let defaults = TestDefaults::new(
        &flat,
        vec![],
        vec![],
        vec![(syn::parse_quote!(E), "message".into())],
    )
    .unwrap();
    defaults
        .fill(&mut flat, &TEST_CODEC, &test_fun!(echo), &Extra::default())
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
            &test_fun!(echo).on_error(test_fun!(other)),
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
        let mut decl = test_fun!(f);
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
            fill(&mut flat, &test_fun!(f).on_error(test_fun!(message))).is_err(),
            "{source}"
        );
    }
}

#[test]
fn result_data_occurrences_are_not_implicitly_error_channels() {
    for source in [
        "pub fn f(x:Result<i64,String>){}",
        "pub fn f(x:Option<&Result<i64,String>>){}",
    ] {
        let mut flat = parse(source);
        fill(&mut flat, &test_fun!(f)).unwrap();
        assert!(flat
            .function("f")
            .unwrap()
            .ret
            .slot
            .as_ref()
            .unwrap()
            .outcomes
            .source_error
            .is_none());
    }
    for (source, recipe) in [
        (
            "pub fn f()->Option<Result<i64,String>>{}",
            TestOutputConfig::Unary {
                operation: TestOutputUnary::Optional,
                child: Box::new(TestOutputConfig::Leaf(TestOutputLeafOp::Value)),
            },
        ),
        (
            "pub struct R{pub value:Result<i64,String>}pub fn f()->R{}",
            TestOutputConfig::Branch {
                operation: TestOutputBranch::OwnedFields,
                children: vec![TestOutputConfig::Leaf(TestOutputLeafOp::Value)],
            },
        ),
    ] {
        let mut flat = parse(source);
        test_bind_output(&mut flat, &TEST_CODEC, "f", &recipe, &Extra::default()).unwrap();
        assert!(flat
            .function("f")
            .unwrap()
            .ret
            .slot
            .as_ref()
            .unwrap()
            .outcomes
            .source_error
            .is_none());
    }
    let mut flat =
        parse("pub fn message(e:String)->String{}pub fn f()->Result<Result<i64,String>,String>{}");
    fill(&mut flat, &test_fun!(f).on_error(test_fun!(message))).unwrap();
    assert!(flat
        .function("f")
        .unwrap()
        .ret
        .slot
        .as_ref()
        .unwrap()
        .outcomes
        .source_error
        .is_some());
    assert!(TestDefaults::new(
        &flat,
        vec![],
        vec![test_expand_return!(Result<i64,String>).field_self().into()],
        vec![]
    )
    .is_ok());
    assert!(TestDefaults::new(
        &flat,
        vec![test_expand_param!(Result<i64,String>).variant_self().into()],
        vec![],
        vec![]
    )
    .is_ok());
}

#[test]
fn result_error_defaults_are_explicit_unique_and_selected_with_helpers() {
    let flat = parse("pub type E=String;pub fn message(e:E)->String{}pub fn f()->Result<i64,E>{}");
    assert!(TestDefaults::new(
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
    let names = test_language::source_items(
        &[],
        &[],
        &errors,
        [&test_fun!(f).on_error(test_fun!(message))],
    );
    assert!(names.contains(&"f".into()) && names.contains(&"message".into()));
}

#[test]
fn selector_schema_separates_control_from_mutually_exclusive_payloads() {
    use prebindgen_flat::WireNode;
    let mut model = parse("pub struct Item {} pub fn empty()->Item{} pub fn numbered(n:i32)->Item{} pub fn take(v:Item){}");
    test_bind_input(
        &mut model,
        &TEST_CODEC,
        "take",
        "v",
        &TestInputConfig::selector([
            TestInputConfig::constructor("empty", []),
            TestInputConfig::constructor(
                "numbered",
                [TestInputConfig::Leaf(TestInputLeafOp::Value)],
            ),
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
            .output_leaf(crate::OutputLeaf::new(
                &parse(""),
                &syn::parse_quote!(i32),
                &Extra::default(),
                syn::parse_quote!(out),
                quote!(value),
                crate::ValueAccess::Owned,
            ))
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
            false,
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
        test_bind_input(
            &mut flat,
            &TEST_CODEC,
            "f",
            "value",
            &TestInputConfig::constructor(
                name,
                vec![TestInputConfig::Leaf(TestInputLeafOp::Value); count],
            ),
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
        test_bind_output(
            &mut flat,
            &TEST_CODEC,
            "f",
            &TestOutputConfig::combine(vec![
                TestOutputConfig::getter(
                    "observe",
                    TestOutputConfig::Leaf(TestOutputLeafOp::Value)
                );
                count
            ]),
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
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "f",
        &TestOutputConfig::Leaf(TestOutputLeafOp::Value),
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
    let error = test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "f",
        &TestOutputConfig::Leaf(TestOutputLeafOp::Value),
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
    let optional = TestInputConfig::Unary {
        operation: TestInputUnary::Optional,
        child: Box::new(TestInputConfig::Leaf(TestInputLeafOp::Value)),
    };
    test_bind_input(
        &mut flat,
        &TEST_CODEC,
        "f",
        "v",
        &optional,
        &Extra::default(),
    )
    .unwrap();
    test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "f",
        &TestOutputConfig::Unary {
            operation: TestOutputUnary::Optional,
            child: Box::new(TestOutputConfig::Leaf(TestOutputLeafOp::Value)),
        },
        &Extra::default(),
    )
    .unwrap();
    let input_before = flat.function("f").unwrap().params[0]
        .ty
        .slot
        .as_ref()
        .unwrap()
        .code
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
            TestInputConfig::selector([
                TestInputConfig::constructor("empty", []),
                TestInputConfig::constructor(
                    "some",
                    [TestInputConfig::Leaf(TestInputLeafOp::Value)],
                ),
            ]),
            "reject_alternative_metadata",
            "alternative metadata rejected",
        ),
    ] {
        let error = test_bind_input(
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
                .code
                .to_string(),
            input_before
        );
    }
    let error = test_bind_output(
        &mut flat,
        &TEST_CODEC,
        "f",
        &TestOutputConfig::Unary {
            operation: TestOutputUnary::Optional,
            child: Box::new(TestOutputConfig::Leaf(TestOutputLeafOp::Value)),
        },
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
fn preparation_fills_one_fixed_schema_and_emission_needs_no_defaults() {
    let source = parse("pub struct R { pub n:i64 } pub fn make(n:i64)->R{} pub fn inspect(r:&R)->i64{} pub fn echo(r:Option<R>)->Option<R>{}");
    let defaults = TestDefaults::new(
        &source,
        vec![test_expand_param!(R).variant(test_fun!(make)).into()],
        vec![test_expand_return!(R).field(test_fun!(inspect)).into()],
        vec![],
    )
    .unwrap();
    let mut source = source;
    let input = defaults
        .resolve_input(
            &source,
            &TEST_CODEC,
            &source.function("echo").unwrap().params[0].ty.ty,
            &TestInputConfig::auto(),
            &Extra::default(),
        )
        .unwrap();
    let output = defaults
        .resolve_output(
            &source,
            &TEST_CODEC,
            &source.function("echo").unwrap().ret.ty,
            &TestOutputConfig::auto(),
            &Extra::default(),
        )
        .unwrap();
    assert!(
        matches!(&input,TestInput::Unary { operation: TestInputUnary::Optional, child } if matches!(child.as_ref(),TestInput::Branch { operation: TestInputBranch::Constructor { function, .. }, .. } if function=="make"))
    );
    assert!(
        matches!(&output,TestOutput::Unary { operation: TestOutputUnary::Optional, child } if matches!(child.as_ref(),TestOutput::Unary { operation: TestOutputUnary::Getter { function, .. }, .. } if function=="inspect"))
    );
    assert!(source.function("echo").unwrap().ret.slot.is_none());
    defaults
        .fill(
            &mut source,
            &TEST_CODEC,
            &test_fun!(echo),
            &Extra::default(),
        )
        .unwrap();
    drop(defaults);
    validate_bindings(&source).unwrap();
    assert!(source.function("echo").unwrap().params[0].ty.slot.is_some());
    assert!(source.function("echo").unwrap().ret.slot.is_some());
    assert!(source.function("make").unwrap().ret.slot.is_none());
}

#[test]
fn sequence_identity_uses_element_defaults_and_staged_errors_name_the_function() {
    let source = model();
    let defaults = TestDefaults::new(
        &source,
        vec![
            test_expand_param!(Payload).variant(test_fun!(make)).into(),
            test_expand_param!(Vec<Payload>).variant_self().into(),
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
            &TestInputConfig::auto(),
            &Extra::default(),
        )
        .unwrap();
    assert!(
        matches!(resolved,TestInput::Unary { operation: TestInputUnary::Sequence, child } if matches!(&*child,TestInput::Branch { operation: TestInputBranch::Constructor { function, .. }, .. } if function=="make"))
    );
    let mut source = source;
    let error = defaults
        .fill(
            &mut source,
            &TEST_CODEC,
            &test_fun!(foo).input("x", TestInputConfig::call("missing")),
            &Extra::default(),
        )
        .unwrap_err();
    assert!(error.starts_with("foo:"), "{error}");
    let mut invalid = parse("pub fn broken()->i64{}");
    let error = defaults
        .fill(
            &mut invalid,
            &TEST_CODEC,
            &test_fun!(broken).output(TestOutputConfig::Unary {
                operation: TestOutputUnary::Optional,
                child: Box::new(TestOutputConfig::Leaf(TestOutputLeafOp::Value)),
            }),
            &Extra::default(),
        )
        .unwrap_err();
    assert!(error.starts_with("broken:"), "{error}");
}

#[test]
fn omitted_typed_fields_drop_all_enclosing_option_and_sequence_wires() {
    let mut model = parse("pub type Hidden=String;pub type Other=u32;pub struct Record{pub optional:Option<Hidden>,pub nested:Option<Vec<Option<Hidden>>>,pub visible:i64}pub fn parts(value:Record)->Record{}pub fn read()->Record{}");
    let output: TestOutputConfig = test_expand_return!(Record)
        .fields_into(
            test_fields!(parts)
                .field("optional", test_expand_return!(Hidden))
                .field_override("nested", test_expand_return!(Hidden)),
        )
        .into();
    test_bind_output(&mut model, &TEST_CODEC, "read", &output, &Extra::default()).unwrap();
    let plan = model.function("read").unwrap().ret.slot.as_ref().unwrap();
    assert_eq!(
        prebindgen_flat::wire_definitions(std::iter::once(&plan.outcomes.success)).count(),
        1
    );
    assert_eq!(plan.values.len(), 1);
    let invalid: TestOutputConfig = test_expand_return!(Record)
        .fields_into(test_fields!(parts).field("optional", test_expand_return!(Other)))
        .into();
    assert!(
        test_bind_output(&mut model, &TEST_CODEC, "read", &invalid, &Extra::default()).is_err()
    );
}

#[test]
fn named_getter_resolution_preserves_the_selected_name_in_the_tree() {
    let model = parse("pub type T=String;pub fn length(value:&T)->i64{}");
    let defaults = TestDefaults::new(&model, vec![], vec![], vec![]).unwrap();
    let tree: TestOutputConfig = test_expand_return!(T)
        .field(test_fun!(length).name("size"))
        .into();
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
        matches!(tree,TestOutput::Unary { operation: TestOutputUnary::Named { name }, child: output,  } if name=="size" && matches!(output.as_ref(),TestOutput::Unary { operation: TestOutputUnary::Getter { function }, child: output,  } if function=="length"&&matches!(output.as_ref(),TestOutput::Leaf(TestOutputLeafOp::Value))))
    );
}

#[test]
fn helper_references_reject_binding_only_decorations() {
    assert!(std::panic::catch_unwind(
        || test_expand_param!(T).variant(test_fun!(make).name("renamed"))
    )
    .is_err());
    assert!(
        std::panic::catch_unwind(|| test_fun!(f).on_error(test_fun!(message).name("renamed")))
            .is_err()
    );
    assert!(std::panic::catch_unwind(|| test_expand_return!(T)
        .field(test_fun!(get).expand_return(test_expand_return!(i64).field_self())))
    .is_err());
}

#[test]
fn nested_requests_resolve_into_strict_schemas_before_compilation() {
    let source = parse("pub type R=String;pub fn make(a:i32,b:Option<i64>)->R{}pub fn narrow(v:i16)->i64{}pub fn run(value:R)->i64{}");
    let defaults = TestDefaults::new(
        &source,
        vec![test_expand_param!(i64).variant(test_fun!(narrow)).into()],
        vec![],
        vec![],
    )
    .unwrap();
    let requested: TestInputConfig = TestInputConfig::constructor(
        "make",
        [
            TestInputConfig::Leaf(TestInputLeafOp::Value),
            TestInputConfig::Unary {
                operation: TestInputUnary::Optional,
                child: Box::new(TestInputConfig::auto()),
            },
        ],
    );
    let resolved: TestInput = defaults
        .resolve_input(
            &source,
            &TEST_CODEC,
            &source.function("run").unwrap().params[0].ty.ty,
            &requested,
            &Extra::default(),
        )
        .unwrap();
    let TestInput::Branch {
        operation: TestInputBranch::Constructor { .. },
        children: args,
        ..
    } = &resolved
    else {
        panic!("constructor not resolved")
    };
    assert!(matches!(&args[0], TestInput::Leaf(TestInputLeafOp::Value)));
    assert!(
        matches!(&args[1],TestInput::Unary { operation: TestInputUnary::Optional, child } if matches!(child.as_ref(),TestInput::Branch { operation: TestInputBranch::Constructor { function }, children: args,  } if function=="narrow"&&matches!(args.as_slice(),[TestInput::Leaf(TestInputLeafOp::Value)])))
    );
    compile_input(
        &source,
        &TEST_CODEC,
        "run",
        "value",
        &resolved,
        &Extra::default(),
    )
    .unwrap();
    assert!(source.function("run").unwrap().params[0].ty.slot.is_none());
}

#[test]
fn completed_subtrees_keep_explicit_choices_when_used_in_preparation() {
    let source=parse("pub type R=String;pub fn direct(v:i64)->R{}pub fn narrow(v:i16)->i64{}pub fn observe(r:&R)->i64{}pub fn sign(v:&i64)->i8{}pub fn run(value:R,count:i64)->R{}");
    let defaults = TestDefaults::new(
        &source,
        vec![test_expand_param!(i64).variant(test_fun!(narrow)).into()],
        vec![test_expand_return!(i64).field(test_fun!(sign)).into()],
        vec![],
    )
    .unwrap();
    let input: TestInput =
        TestInput::constructor("direct", [TestInput::Leaf(TestInputLeafOp::Value)]);
    let output: TestOutput =
        TestOutput::getter("observe", TestOutput::Leaf(TestOutputLeafOp::Value));
    let decl = test_fun!(run)
        .input("value", input.map_into())
        .output(output.map_into());
    let run = source.function("run").unwrap();
    let input = defaults
        .resolve_input(
            &source,
            &TEST_CODEC,
            &run.params[0].ty.ty,
            &decl.params["value"],
            &Extra::default(),
        )
        .unwrap();
    let count = defaults
        .resolve_input(
            &source,
            &TEST_CODEC,
            &run.params[1].ty.ty,
            &TestInputConfig::auto(),
            &Extra::default(),
        )
        .unwrap();
    let output = defaults
        .resolve_output(
            &source,
            &TEST_CODEC,
            &run.ret.ty,
            decl.ret.as_ref().unwrap(),
            &Extra::default(),
        )
        .unwrap();
    assert!(
        matches!(&input,TestInput::Branch { operation: TestInputBranch::Constructor { function }, children: args,  } if function=="direct"&&matches!(args.as_slice(),[TestInput::Leaf(TestInputLeafOp::Value)]))
    );
    assert!(
        matches!(&count,TestInput::Branch { operation: TestInputBranch::Constructor { function, .. }, .. } if function=="narrow")
    );
    assert!(
        matches!(&output,TestOutput::Unary { operation: TestOutputUnary::Getter { function }, child: output,  } if function=="observe"&&matches!(output.as_ref(),TestOutput::Leaf(TestOutputLeafOp::Value)))
    );
    let mut source = source;
    defaults
        .fill(&mut source, &TEST_CODEC, &decl, &Extra::default())
        .unwrap();
}

#[test]
fn result_resolution_consumes_nested_requests_and_preserves_error_mapper() {
    let source = parse("pub type E=String;pub fn message(e:&E)->String{}pub fn sign(v:&i64)->i8{}pub fn run()->Result<Option<i64>,E>{}");
    let defaults = TestDefaults::new(
        &source,
        vec![],
        vec![test_expand_return!(i64).field(test_fun!(sign)).into()],
        vec![],
    )
    .unwrap();
    let request = TestOutputConfig::Unary {
        operation: TestOutputUnary::Result {
            error: Some("message".into()),
        },
        child: Box::new(TestOutputConfig::Unary {
            operation: TestOutputUnary::Optional,
            child: Box::new(TestOutputConfig::auto()),
        }),
    };
    let resolved: TestOutput = defaults
        .resolve_output(
            &source,
            &TEST_CODEC,
            &source.function("run").unwrap().ret.ty,
            &request,
            &Extra::default(),
        )
        .unwrap();
    let TestOutput::Unary {
        operation: TestOutputUnary::Result { error },
        child: ok,
    } = &resolved
    else {
        panic!("missing Result")
    };
    assert_eq!(error.as_deref(), Some("message"));
    assert!(
        matches!(ok.as_ref(),TestOutput::Unary { operation: TestOutputUnary::Optional, child } if matches!(child.as_ref(),TestOutput::Unary { operation: TestOutputUnary::Getter { function }, child: output,  } if function=="sign" && matches!(output.as_ref(),TestOutput::Leaf(TestOutputLeafOp::Value))))
    );
    compile_output(&source, &TEST_CODEC, "run", &resolved, &Extra::default()).unwrap();
    let mut source = source;
    defaults
        .fill(
            &mut source,
            &TEST_CODEC,
            &test_fun!(run)
                .output(TestOutputConfig::Unary {
                    operation: TestOutputUnary::Optional,
                    child: Box::new(TestOutputConfig::auto()),
                })
                .on_error(test_fun!(message)),
            &Extra::default(),
        )
        .unwrap();
    assert!(source
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
    let defaults = TestDefaults::new(&source, vec![], vec![], vec![]).unwrap();
    let tree = TestOutputConfig::Branch {
        operation: TestOutputBranch::OwnedFields,
        children: vec![
            TestOutputConfig::for_type(
                syn::parse_quote!(Hidden),
                TestOutputConfig::Leaf(TestOutputLeafOp::Discard),
            ),
            TestOutputConfig::Unary {
                operation: TestOutputUnary::Sequence {
                    delivery: SequenceDelivery::Callback,
                    mode: Some(ProjectionMode::Owned),
                },
                child: Box::new(TestOutputConfig::auto()),
            },
        ],
    };
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
        matches!(&resolved,TestOutput::Branch { operation: TestOutputBranch::OwnedFields, children: fields } if matches!(fields.as_slice(),[TestOutput::Leaf(TestOutputLeafOp::Discard),TestOutput::Unary { operation: TestOutputUnary::Sequence { delivery: SequenceDelivery::Callback, mode: Some(ProjectionMode::Owned), .. }, .. }]))
    );
    let round_trip = defaults
        .resolve_output(
            &source,
            &TEST_CODEC,
            &source.function("read").unwrap().ret.ty,
            &resolved.clone().map_into(),
            &Extra::default(),
        )
        .unwrap();
    assert_eq!(format!("{resolved:?}"), format!("{round_trip:?}"));
}

#[test]
fn bulk_normalization_preserves_callback_boundaries_inside_structural_nodes() {
    let flat = parse("pub fn values()->Vec<Vec<i64>>{}");
    let defaults = TestDefaults::new(&flat, vec![], vec![], vec![]).unwrap();
    let callback = TestOutputConfig::Unary {
        operation: TestOutputUnary::Sequence {
            delivery: SequenceDelivery::Callback,
            mode: None,
        },
        child: Box::new(TestOutputConfig::Leaf(TestOutputLeafOp::Value).bulk()),
    };
    let recipe = TestOutputConfig::Unary {
        operation: TestOutputUnary::Sequence {
            delivery: SequenceDelivery::Value,
            mode: None,
        },
        child: Box::new(callback),
    };
    let resolved = defaults
        .resolve_output(
            &flat,
            &TEST_CODEC,
            &flat.function("values").unwrap().ret.ty,
            &recipe,
            &Extra::default(),
        )
        .unwrap();
    assert!(
        matches!(resolved,TestOutput::Unary { operation: TestOutputUnary::Sequence { .. }, child: element, .. } if matches!(*element,TestOutput::Unary { operation: TestOutputUnary::Sequence { delivery: SequenceDelivery::Callback, .. }, .. }))
    );
}

#[test]
fn unsupported_bulk_is_diagnosed_before_adapter_lowering() {
    let flat = parse("pub fn values()->Option<Vec<i64>>{}");
    let recipe = TestOutput::Unary {
        operation: TestOutputUnary::Optional,
        child: Box::new(TestOutput::Unary {
            operation: TestOutputUnary::Sequence {
                delivery: SequenceDelivery::Callback,
                mode: None,
            },
            child: Box::new(TestOutput::Leaf(TestOutputLeafOp::Value)),
        }),
    }
    .bulk();
    let error = compile_output(&flat, &TEST_CODEC, "values", &recipe, &Extra::default())
        .err()
        .unwrap();
    assert!(
        error.contains("bulk outputs are unsupported by this codec"),
        "{error}"
    );
    assert!(flat.function("values").unwrap().ret.slot.is_none());
}

#[test]
fn optional_fields_have_one_continuation_and_linear_generated_code_size() {
    let size = |width: usize| {
        let fields = (0..width)
            .map(|i| format!("pub f{i}:Option<i64>"))
            .collect::<Vec<_>>()
            .join(",");
        let flat = parse(&format!(
            "pub struct Wide{{{fields}}} pub fn consume(value:Wide){{}}"
        ));
        let recipe = TestInput::Branch {
            operation: TestInputBranch::Fields,
            children: (0..width)
                .map(|_| TestInput::Unary {
                    operation: TestInputUnary::Optional,
                    child: Box::new(TestInput::Leaf(TestInputLeafOp::Value)),
                })
                .collect(),
        };
        let plan = compile_input(
            &flat,
            &TEST_CODEC,
            "consume",
            "value",
            &recipe,
            &Extra::default(),
        )
        .unwrap();
        let code = plan.code.to_string();
        assert_eq!(code.matches("continuation").count(), 1);
        code.len()
    };
    let small = size(8);
    let large = size(64);
    assert!(large < small * 10, "{small} -> {large}");
}

#[test]
fn optional_temporary_borrows_survive_through_the_next_parameter_and_source_call() {
    let flat = parse("pub fn combine(first:Option<&str>,second:Option<&str>)->usize{}");
    let recipe = TestInput::Unary {
        operation: TestInputUnary::Optional,
        child: Box::new(TestInput::Unary {
            operation: TestInputUnary::Temporary,
            child: Box::new(TestInput::Leaf(TestInputLeafOp::Value)),
        }),
    };
    let first = compile_input(
        &flat,
        &TEST_CODEC,
        "combine",
        "first",
        &recipe,
        &Extra::default(),
    )
    .unwrap();
    let second = compile_input(
        &flat,
        &TEST_CODEC,
        "combine",
        "second",
        &recipe,
        &Extra::default(),
    )
    .unwrap();
    let a = syn::parse_quote!(first);
    let b = syn::parse_quote!(second);
    let body = second.bind(&b, quote!(Ok(combine(#a,#b)))).unwrap();
    let body = first.bind(&a, body).unwrap();
    let parameters = TestAbi::wires(&first.node)
        .chain(TestAbi::wires(&second.node))
        .map(|w| {
            let name = &w.name;
            let ty = w.rust_type();
            quote!(#name:#ty)
        })
        .collect::<Vec<_>>();
    let program = quote! {
        fn decode(_: &str,value:String)->Result<String,String>{Ok(value)}
        fn combine(first:Option<&str>,second:Option<&str>)->usize {first.map_or(0,str::len)+second.map_or(0,str::len)}
        fn bridge(#(#parameters),*)->Result<usize,String>{#body}
        fn main(){assert_eq!(bridge(1,"hello".into(),1,"world".into()).unwrap(),10);assert_eq!(bridge(0,"ignored".into(),1,"a".into()).unwrap(),1);assert_eq!(bridge(1,"a".into(),0,"ignored".into()).unwrap(),1);}
    };
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("borrows.rs");
    let executable = directory.path().join("borrows");
    std::fs::write(&source, program.to_string()).unwrap();
    let built = std::process::Command::new("rustc")
        .args(["--edition=2021"])
        .arg(source)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert!(std::process::Command::new(executable)
        .status()
        .unwrap()
        .success());
}
