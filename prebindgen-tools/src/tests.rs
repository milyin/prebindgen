use super::*;
struct TestCodec;
impl Codec for TestCodec {
    fn decode_text(&self, w: &syn::Ident) -> TokenStream {
        quote!(decode_text(#w)?)
    }
    fn decode_bytes(&self, w: &syn::Ident) -> TokenStream {
        quote!(decode_bytes(#w)?)
    }
}
fn model() -> Flat<Slot> {
    let file=syn::parse_file("pub struct Payload { pub data: Vec<u8> } pub fn make(s: String) -> Payload {} pub fn foo(x: Payload, y: Payload) -> Payload {}").unwrap();
    Flat::parse(file.items.into_iter().map(|i| (i, Default::default()))).unwrap()
}
#[test]
fn different_occurrences_have_different_converters() {
    let mut flat = model();
    let f = flat.function_mut("foo").unwrap();
    f.params[0].ty.slot = Slot::Input(Input::selector([
        Input::Handle,
        Input::constructor("make", [Input::Text]),
    ]));
    f.params[1].ty.slot = Slot::Input(Input::Handle);
    f.ret.slot = Slot::Output(Output::Handle);
    let ready = prepare(flat, &TestCodec).unwrap();
    let f = ready.function("foo").unwrap();
    let Prepared::Input(x) = &f.params[0].ty.slot else {
        panic!()
    };
    let Prepared::Input(y) = &f.params[1].ty.slot else {
        panic!()
    };
    assert_eq!(x.wires.len(), 3);
    assert_eq!(y.wires.len(), 1);
}
#[test]
fn rejects_incomplete_slots() {
    let mut f = model();
    f.function_mut("foo").unwrap().ret.slot = Slot::Output(Output::Handle);
    assert!(prepare(f, &TestCodec).unwrap_err().contains("no input"));
}
#[test]
fn rejects_constructor_arity_mismatch() {
    let mut f = model();
    let foo = f.function_mut("foo").unwrap();
    foo.params[0].ty.slot = Slot::Input(Input::constructor("foo", []));
    foo.params[1].ty.slot = Slot::Input(Input::Handle);
    foo.ret.slot = Slot::Output(Output::Handle);
    assert!(prepare(f, &TestCodec)
        .unwrap_err()
        .contains("argument count"));
}
#[test]
fn rejects_double_move_and_field_move() {
    for output in [
        Output::combine([Output::Handle, Output::Handle]),
        Output::fields([Output::Handle]),
    ] {
        let mut f = model();
        let foo = f.function_mut("foo").unwrap();
        for p in &mut foo.params {
            p.ty.slot = Slot::Input(Input::Handle);
        }
        foo.ret.slot = Slot::Output(output);
        assert!(prepare(f, &TestCodec).is_err());
    }
}
#[test]
fn observations_precede_move_but_wires_keep_order() {
    let mut f = model();
    let foo = f.function_mut("foo").unwrap();
    for p in &mut foo.params {
        p.ty.slot = Slot::Input(Input::Handle);
    }
    foo.ret.slot = Slot::Output(Output::combine([
        Output::Handle,
        Output::fields([Output::Bytes]),
    ]));
    let ready = prepare(f, &TestCodec).unwrap();
    let Prepared::Output(o) = &ready.function("foo").unwrap().ret.slot else {
        panic!()
    };
    assert!(matches!(o.wires[0].kind, WireKind::Handle(_)));
    let body = o.statements.to_string();
    assert!(body.find("clone").unwrap() < body.find("insert").unwrap());
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
        let mut flat = Flat::parse(
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
    let declarations = Declarations::new(
        &flat,
        vec![expand_param!(Payload)
            .variant(fun!(make))
            .variant_self()
            .into()],
    )
    .unwrap();
    declarations
        .fill(
            &mut flat,
            &fun!(foo).expand_param("y", expand_param!(Payload).variant_self()),
        )
        .unwrap();
    let ready = prepare(flat, &TestCodec).unwrap();
    let f = ready.function("foo").unwrap();
    let Prepared::Input(x) = &f.params[0].ty.slot else {
        panic!()
    };
    let Prepared::Input(y) = &f.params[1].ty.slot else {
        panic!()
    };
    assert_eq!(x.wires.len(), 3);
    assert_eq!(y.wires.len(), 1);
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
    let ready = prepare(flat, &TestCodec).unwrap();
    let Prepared::Output(output) = &ready.function("read").unwrap().ret.slot else {
        panic!()
    };
    assert!(matches!(output.wires[0].kind, WireKind::Text));
    assert!(matches!(output.wires[1].kind, WireKind::Bytes));
}
