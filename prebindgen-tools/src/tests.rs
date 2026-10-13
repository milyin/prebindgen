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
#[derive(Clone, Debug, Default)]
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
    ) -> Result<PreparedInput<Self::Wire>> {
        Ok(PreparedInput {
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
    ) -> Result<PreparedOutput<Self::Wire>> {
        if borrowed && !extra.copy {
            return Err("test token cannot be copied".into());
        }
        Ok(PreparedOutput {
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
fn model() -> Configured<Extra> {
    let file=syn::parse_file("pub struct Payload { pub data: Vec<u8> } pub fn make(s: String) -> Payload {} pub fn foo(x: Payload, y: Payload) -> Payload {}").unwrap();
    Flat::parse(file.items.into_iter().map(|i| (i, Default::default()))).unwrap()
}
#[test]
fn different_occurrences_have_different_converters() {
    let mut flat = model();
    let f = flat.function_mut("foo").unwrap();
    f.params[0].ty.slot.recipe = Some(Input::selector([
        Input::Foreign,
        Input::constructor("make", [Input::Text]),
    ]));
    f.params[1].ty.slot.recipe = Some(Input::Foreign);
    f.ret.slot.recipe = Some(Output::Foreign);
    let ready = prepare(flat, &TEST_CODEC).unwrap();
    let f = ready.function("foo").unwrap();
    let Some(x) = &f.params[0].ty.slot else {
        panic!()
    };
    let Some(y) = &f.params[1].ty.slot else {
        panic!()
    };
    assert_eq!(x.wires.len(), 3);
    assert_eq!(y.wires.len(), 1);
}
#[test]
fn rejects_incomplete_slots() {
    let mut f = model();
    f.function_mut("foo").unwrap().ret.slot.recipe = Some(Output::Foreign);
    assert!(prepare(f, &TEST_CODEC).unwrap_err().contains("no input"));
}
#[test]
fn rejects_inputs_without_a_result_recipe() {
    let mut flat = model();
    flat.function_mut("foo").unwrap().params[0].ty.slot.recipe = Some(Input::Foreign);
    assert!(prepare(flat, &TEST_CODEC)
        .unwrap_err()
        .contains("no output converter"));
}

#[test]
fn unbound_helpers_remain_source_only_and_records_survive_preparation() {
    let mut flat = model();
    let record = flat.records.values().next().unwrap().clone();
    Declarations::new(&flat, vec![])
        .unwrap()
        .fill(&mut flat, &fun!(foo))
        .unwrap();
    let prepared = prepare(flat, &TEST_CODEC).unwrap();
    let helper = prepared.function("make").unwrap();
    assert!(helper.ret.slot.is_none());
    assert!(helper.params.iter().all(|p| p.ty.slot.is_none()));
    let retained = prepared.record(&record.ty).unwrap();
    assert_eq!(retained.ty, record.ty);
    assert_eq!(retained.fields[0].name, record.fields[0].name);
    assert_eq!(retained.fields[0].ty, record.fields[0].ty);
}

#[test]
fn rejects_constructor_arity_mismatch() {
    let mut f = model();
    let foo = f.function_mut("foo").unwrap();
    foo.params[0].ty.slot.recipe = Some(Input::constructor("foo", []));
    foo.params[1].ty.slot.recipe = Some(Input::Foreign);
    foo.ret.slot.recipe = Some(Output::Foreign);
    assert!(prepare(f, &TEST_CODEC)
        .unwrap_err()
        .contains("argument count"));
}
#[test]
fn rejects_double_move_and_field_move() {
    for output in [
        Output::combine([Output::Foreign, Output::Foreign]),
        Output::fields([Output::Foreign]),
    ] {
        let mut f = model();
        let foo = f.function_mut("foo").unwrap();
        for p in &mut foo.params {
            p.ty.slot.recipe = Some(Input::Foreign);
        }
        foo.ret.slot.recipe = Some(output);
        assert!(prepare(f, &TEST_CODEC).is_err());
    }
}
#[test]
fn observations_precede_move_but_wires_keep_order() {
    let mut f = model();
    let foo = f.function_mut("foo").unwrap();
    for p in &mut foo.params {
        p.ty.slot.recipe = Some(Input::Foreign);
    }
    foo.ret.slot.recipe = Some(Output::combine([
        Output::Foreign,
        Output::fields([Output::Bytes]),
    ]));
    let ready = prepare(f, &TEST_CODEC).unwrap();
    let Some(o) = &ready.function("foo").unwrap().ret.slot else {
        panic!()
    };
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
        let mut flat = Configured::<Extra>::parse(
            syn::parse_file(source)
                .unwrap()
                .items
                .into_iter()
                .map(|i| (i, Default::default())),
        )
        .unwrap();
        let declarations = Declarations::new(&flat, vec![]).unwrap();
        let function = fun!(read)
            .expand_return(expand_return!(Payload).fields(fields!(parts).field(field, output)));
        assert!(declarations
            .fill(&mut flat, &function)
            .unwrap_err()
            .contains(message));
    }
}
#[test]
fn declarations_preserve_defaults_and_occurrence_overrides() {
    let mut flat = model();
    let f = flat.function_mut("foo").unwrap();
    f.params[0].ty.slot.extra.name = "x";
    f.params[1].ty.slot.extra.name = "y";
    f.ret.slot.extra.name = "result";
    let declarations = Declarations::new(
        &flat,
        vec![expand_param!(Payload)
            .variant(fun!(make))
            .variant_self()
            .into()],
    )
    .unwrap();
    declarations.fill(&mut flat, &fun!(foo)).unwrap();
    declarations
        .fill(
            &mut flat,
            &fun!(foo).expand_param("y", expand_param!(Payload).variant_self()),
        )
        .unwrap();
    let ready = prepare(flat, &TEST_CODEC).unwrap();
    let f = ready.function("foo").unwrap();
    let Some(x) = &f.params[0].ty.slot else {
        panic!()
    };
    let Some(y) = &f.params[1].ty.slot else {
        panic!()
    };
    assert_eq!(x.wires.len(), 3);
    assert_eq!(y.wires.len(), 1);
    assert!(matches!(x.wires[2].kind, WireKind::Foreign));
    assert_eq!(x.wires[2].extra.name, "x");
    assert_eq!(x.wires[0].extra.name, "x");
    assert_eq!(x.wires[1].extra.name, "x");
    assert!(matches!(y.wires[0].kind, WireKind::Foreign));
    assert_eq!(y.wires[0].extra.name, "y");
    assert!(x.validation.to_string().contains("\"x\""));
    let Some(result) = &f.ret.slot else { panic!() };
    assert!(matches!(result.wires[0].kind, WireKind::Foreign));
    assert_eq!(result.wires[0].extra.name, "result");
}
#[test]
fn field_defaults_follow_type_expansions() {
    let source = "pub type Key=String; pub struct Payload { pub key: Key, pub data: Vec<u8> } pub fn parts(p: &Payload) -> Payload {} pub fn key_text(k: &Key) -> String {} pub fn read(p: Payload) -> Payload {}";
    let mut flat = Flat::parse(
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default())),
    )
    .unwrap();
    let declarations = Declarations::new(
        &flat,
        vec![expand_return!(Key).field(fun!(key_text)).into()],
    )
    .unwrap();
    declarations
        .fill(
            &mut flat,
            &fun!(read).expand_return(expand_return!(Payload).fields(fields!(parts))),
        )
        .unwrap();
    let ready = prepare(flat, &TEST_CODEC).unwrap();
    let Some(output) = &ready.function("read").unwrap().ret.slot else {
        panic!()
    };
    assert!(matches!(output.wires[0].kind, WireKind::Text));
    assert!(matches!(output.wires[1].kind, WireKind::Bytes));
    assert_eq!(output.wires[0].extra.ty, "String");
    assert_eq!(output.wires[1].extra.ty, "Vec < u8 >");
}

#[test]
fn explicit_scalar_identity_is_a_value_conversion() {
    let items = syn::parse_file("pub fn f(x:i64)->i64{}").unwrap().items;
    let mut flat =
        Configured::<Extra>::parse(items.into_iter().map(|i| (i, Default::default()))).unwrap();
    let f = flat.function_mut("f").unwrap();
    f.params[0].ty.slot.extra.name = "scalar-input";
    f.ret.slot.extra.name = "scalar-output";
    let defaults = Declarations::new(
        &flat,
        vec![
            expand_param!(i64).variant_self().into(),
            expand_return!(i64).field_self().into(),
        ],
    )
    .unwrap();
    defaults.fill(&mut flat, &fun!(f)).unwrap();
    let prepared = prepare(flat, &TEST_CODEC).unwrap();
    let f = prepared.function("f").unwrap();
    let Some(input) = &f.params[0].ty.slot else {
        panic!()
    };
    let Some(output) = &f.ret.slot else { panic!() };
    assert!(matches!(
        input.wires[0].kind,
        WireKind::Scalar(Primitive::I64)
    ));
    assert!(matches!(
        output.wires[0].kind,
        WireKind::Scalar(Primitive::I64)
    ));
    assert_eq!(input.wires[0].extra.name, "scalar-input");
    assert_eq!(input.wires[0].extra.ty, "i64");
    assert_eq!(output.wires[0].extra.name, "scalar-output");
    assert_eq!(output.wires[0].extra.ty, "i64");
    assert!(input.validation.to_string().contains("scalar-input"));
    assert!(output.statements.to_string().contains("scalar-output"));
}

#[test]
fn nested_recipes_use_the_boundary_occurrences_extra() {
    let items = syn::parse_file("pub type Key=String; pub struct Payload { pub key:Key } pub fn make(k:Key)->Payload {} pub fn parts(p:&Payload)->Payload {} pub fn foo(x:Payload,y:Payload)->Payload {}").unwrap().items;
    let mut flat =
        Configured::<Extra>::parse(items.into_iter().map(|i| (i, Default::default()))).unwrap();
    flat.function_mut("make").unwrap().params[0]
        .ty
        .slot
        .extra
        .name = "constructor declaration";
    flat.function_mut("parts").unwrap().ret.slot.extra.name = "getter declaration";
    let f = flat.function_mut("foo").unwrap();
    f.params[0].ty.slot = Slot {
        recipe: Some(Input::selector([
            Input::Foreign,
            Input::constructor("make", [Input::Foreign]),
        ])),
        extra: Extra {
            name: "x",
            copy: false,
        },
    };
    f.params[1].ty.slot = Slot {
        recipe: Some(Input::fields([Input::Foreign])),
        extra: Extra {
            name: "y",
            copy: false,
        },
    };
    f.ret.slot = Slot {
        recipe: Some(Output::combine([
            Output::Foreign,
            Output::getter("parts", Output::fields([Output::Foreign])),
        ])),
        extra: Extra {
            name: "result",
            copy: true,
        },
    };
    let ready = prepare(flat, &TEST_CODEC).unwrap();
    let f = ready.function("foo").unwrap();
    let Some(x) = &f.params[0].ty.slot else {
        panic!()
    };
    let Some(y) = &f.params[1].ty.slot else {
        panic!()
    };
    let Some(result) = &f.ret.slot else { panic!() };
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
        foreign(&x.wires),
        vec![
            ("x", "crate :: Payload".into()),
            ("x", "crate :: Key".into())
        ]
    );
    assert_eq!(foreign(&y.wires), vec![("y", "crate :: Key".into())]);
    assert_eq!(
        foreign(&result.wires),
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
        let f = flat.function_mut("foo").unwrap();
        for p in &mut f.params {
            p.ty.slot.recipe = Some(Input::Foreign);
        }
        f.ret.slot.recipe = Some(recipe.clone());
        f.ret.slot.extra.copy = copy;
        let prepared = prepare(flat, &TEST_CODEC);
        if copy {
            assert!(prepared.is_ok(), "{prepared:?}");
        } else {
            assert!(prepared
                .unwrap_err()
                .contains("consume the same value twice"));
        }
    }
}

#[test]
fn adapter_can_use_unit_data_even_for_foreign_wires() {
    let mut flat = model();
    let declarations = Declarations::new(&flat, vec![]).unwrap();
    declarations.fill(&mut flat, &fun!(foo)).unwrap();
    let codec = TestCodec::<()>(std::marker::PhantomData);
    let ready = prepare(flat, &codec).unwrap();
    let f = ready.function("foo").unwrap();
    let Some(output) = &f.ret.slot else { panic!() };
    let wire: &Wire<()> = &output.wires[0];
    assert!(matches!(wire.kind, WireKind::Foreign));
    for p in &f.params {
        let Some(input) = &p.ty.slot else { panic!() };
        assert!(matches!(input.wires[0].kind, WireKind::Foreign));
    }
}
