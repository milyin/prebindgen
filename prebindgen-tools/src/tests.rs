use super::*;
struct TestCodec<W>(std::marker::PhantomData<W>);
const TEST_CODEC: TestCodec<WireExtra> = TestCodec(std::marker::PhantomData);
#[derive(Clone, Debug)]
struct WireExtra {
    name: &'static str,
    ty: String,
}
impl From<WireExtra> for () {
    fn from(_: WireExtra) {}
}
// Construction-only data needs no Clone or Debug implementation.
#[derive(Default)]
struct Extra {
    name: &'static str,
    copy: bool,
}
impl<W: From<WireExtra> + Clone + std::fmt::Debug> Codec for TestCodec<W> {
    type Extra = Extra;
    type Wire = W;
    fn wire_extra(&self, ty: &Type, _: &WireKind, extra: &Extra) -> W {
        WireExtra {
            name: extra.name,
            ty: type_key(ty),
        }
        .into()
    }
    fn consumes(&self, _: &Type, extra: &Extra) -> bool {
        !extra.copy
    }
    fn input_foreign(
        &self,
        ty: &Type,
        extra: &Extra,
        w: syn::Ident,
        v: syn::Ident,
        active: &Condition,
    ) -> Result<InputConversion<Self::Wire>> {
        Ok(InputConversion {
            wires: vec![Wire {
                name: w.clone(),
                kind: WireKind::Foreign,
                extra: self.wire_extra(ty, &WireKind::Foreign, extra),
                active: active.clone(),
            }],
            validation: quote!(),
            acquire: quote!(),
            resources: vec![],
            statements: quote!(let #v=receive_token(#w);),
            argument: quote!(#v),
        })
    }
    fn output_foreign(
        &self,
        ty: &Type,
        extra: &Extra,
        w: syn::Ident,
        v: TokenStream,
        borrowed: bool,
    ) -> Result<OutputConversion<Self::Wire>> {
        if borrowed && !extra.copy {
            return Err("test token cannot be copied".into());
        }
        Ok(OutputConversion {
            wires: vec![Wire {
                name: w.clone(),
                kind: WireKind::Foreign,
                extra: self.wire_extra(ty, &WireKind::Foreign, extra),
                active: Condition::default(),
            }],
            statements: quote!(let #w=send_token(#v);),
            values: vec![quote!(#w)],
            commit: quote!(),
        })
    }
    fn decode_scalar(&self, _: Primitive, w: &syn::Ident, extra: &Extra) -> TokenStream {
        let name = extra.name;
        quote!(decode_scalar(#name, #w))
    }
    fn encode_scalar(&self, _: Primitive, v: TokenStream, extra: &Extra) -> TokenStream {
        let name = extra.name;
        quote!(encode_scalar(#name, #v))
    }
    fn decode_text(&self, w: &syn::Ident, _ty: &syn::Type, extra: &Extra) -> TokenStream {
        let name = extra.name;
        quote!(decode_text(#name, #w)?)
    }
    fn decode_bytes(&self, w: &syn::Ident, _ty: &syn::Type, extra: &Extra) -> TokenStream {
        let name = extra.name;
        quote!(decode_bytes(#name, #w)?)
    }
}
fn parse<W>(source: &str) -> Bindings<W> {
    Flat::parse(
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap()
}
fn model<W>() -> Bindings<W> {
    parse("pub struct Payload { pub data: Vec<u8> } pub fn make(s: String) -> Payload {} pub fn foo(x: Payload, y: Payload) -> Payload {}")
}
fn extra(name: &'static str) -> Extra {
    Extra { name, copy: false }
}
fn fill(flat: &mut Bindings<WireExtra>, decl: &FunctionDecl) -> Result<()> {
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
        &Input::selector([Input::Foreign, Input::constructor("make", [Input::Text])]),
        &extra("x"),
    )
    .unwrap();
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "y",
        &Input::Foreign,
        &extra("y"),
    )
    .unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &Output::Foreign,
        &extra("result"),
    )
    .unwrap();
    validate_bindings(&flat).unwrap();
    let f = flat.function("foo").unwrap();
    let x = f.params[0].ty.slot.as_ref().unwrap();
    let y = f.params[1].ty.slot.as_ref().unwrap();
    assert_eq!(x.wires.len(), 3);
    assert_eq!(y.wires.len(), 1);
    assert!(x.wires.iter().all(|w| w.extra.name == "x"));
    assert_eq!(y.wires[0].extra.name, "y");
}
#[test]
fn rejects_incomplete_slots() {
    let mut flat = model();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &Output::Foreign,
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
        &Input::Foreign,
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
        Output::combine([Output::Foreign, Output::Foreign]),
        Output::fields([Output::Foreign]),
    ] {
        assert!(bind_output(&mut flat, &TEST_CODEC, "foo", &output, &Extra::default()).is_err());
        assert_eq!(format!("{flat:?}"), before);
    }
    assert!(bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "missing",
        &Input::Foreign,
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
        &Output::combine([Output::Foreign, Output::fields([Output::Bytes])]),
        &Extra::default(),
    )
    .unwrap();
    let o = flat.function("foo").unwrap().ret.slot.as_ref().unwrap();
    assert!(matches!(o.wires[0].kind, WireKind::Foreign));
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
        &Input::selector([Input::constructor("make", [Input::Text]), Input::Foreign]),
        &extra("x"),
    )
    .unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &Output::Foreign,
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
    assert!(matches!(x.wires[2].kind, WireKind::Foreign));
    assert!(x.wires.iter().all(|w| w.extra.name == "x"));
    assert_eq!(y.wires.len(), 1);
    assert_eq!(y.wires[0].extra.name, "override");
    assert_eq!(f.ret.slot.as_ref().unwrap().wires[0].extra.name, "result");
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
                &Input::Foreign,
                &Extra::default(),
            )
            .unwrap();
        }
        bind_output(
            model,
            &TEST_CODEC,
            "foo",
            &Output::Foreign,
            &Extra::default(),
        )
        .unwrap();
    }
    assert_eq!(format!("{flat:?}"), format!("{reversed:?}"));
    let old_y = format!("{:?}", flat.function("foo").unwrap().params[1].ty.slot);
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &Input::selector([Input::Foreign, Input::constructor("make", [Input::Text])]),
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
    assert!(matches!(output.wires[0].kind, WireKind::Text));
    assert!(matches!(output.wires[1].kind, WireKind::Bytes));
    assert_eq!(output.wires[0].extra.ty, "String");
    assert_eq!(output.wires[1].extra.ty, "Vec < u8 >");
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
    assert!(matches!(
        input.wires[0].kind,
        WireKind::Scalar(Primitive::I64)
    ));
    assert!(matches!(
        output.wires[0].kind,
        WireKind::Scalar(Primitive::I64)
    ));
    assert_eq!(input.wires[0].extra.name, "scalar");
    assert_eq!(input.wires[0].extra.ty, "i64");
    assert_eq!(output.wires[0].extra.name, "scalar");
    assert_eq!(output.wires[0].extra.ty, "i64");
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
        &Input::Foreign,
        &extra("constructor declaration"),
    )
    .unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "parts",
        &Output::Foreign,
        &extra("getter declaration"),
    )
    .unwrap();
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "x",
        &Input::selector([Input::Foreign, Input::constructor("make", [Input::Foreign])]),
        &extra("x"),
    )
    .unwrap();
    bind_input(
        &mut flat,
        &TEST_CODEC,
        "foo",
        "y",
        &Input::fields([Input::Foreign]),
        &extra("y"),
    )
    .unwrap();
    bind_output(
        &mut flat,
        &TEST_CODEC,
        "foo",
        &Output::combine([
            Output::Foreign,
            Output::getter("parts", Output::fields([Output::Foreign])),
        ]),
        &Extra {
            name: "result",
            copy: true,
        },
    )
    .unwrap();
    let f = flat.function("foo").unwrap();
    let foreign = |wires: &[Wire<WireExtra>]| {
        wires
            .iter()
            .filter_map(|w| match &w.kind {
                WireKind::Foreign => Some((w.extra.name, w.extra.ty.clone())),
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
    let recipe = Output::combine([Output::Foreign, Output::Foreign]);
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
fn adapter_can_use_unit_data_even_for_foreign_wires() {
    let mut flat = model();
    let codec = TestCodec::<()>(std::marker::PhantomData);
    Declarations::new(&flat, vec![])
        .unwrap()
        .fill(&mut flat, &codec, &fun!(foo), &Extra::default())
        .unwrap();
    let f = flat.function("foo").unwrap();
    let wire: &Wire<()> = &f.ret.slot.as_ref().unwrap().wires[0];
    assert!(matches!(wire.kind, WireKind::Foreign));
    for p in &f.params {
        assert!(matches!(
            p.ty.slot.as_ref().unwrap().wires[0].kind,
            WireKind::Foreign
        ));
    }
}
