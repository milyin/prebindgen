use super::*;
#[derive(Clone, Debug, PartialEq)]
struct TestInput(u8);
#[derive(Clone, Debug, PartialEq)]
struct TestOutput(u32);
struct TestSchema;
impl Schema for TestSchema {
    type Input = TestInput;
    type Output = TestOutput;
    type Module = ();
    type Function = ();
    type Type = ();
    type Field = ();
    type Variant = ();
}
fn parse(text: &str) -> Result<Flat<TestSchema>> {
    let items = syn::parse_file(text).unwrap().items;
    Flat::parse(items.into_iter().map(|i| {
        (
            i,
            SourceLocation {
                crate_name: Some("source-crate".into()),
                ..Default::default()
            },
        )
    }))
}
#[test]
fn qualifies_names_without_merging_occurrences() {
    let flat = parse("pub type Key = crate::Inner; pub fn f(x:Key,y:Key)->Key{}").unwrap();
    let mut flat = flat
        .map_slots(
            |_, _| Ok(Some(TestInput(0))),
            |_, _| Ok(Some(TestOutput(0))),
        )
        .unwrap();
    let f = flat.function_mut("f").unwrap();
    f.params[0].ty.slot = Some(TestInput(1));
    f.params[1].ty.slot = Some(TestInput(2));
    f.ret.slot = Some(TestOutput(3));
    assert_eq!(f.params[0].ty.slot, Some(TestInput(1)));
    assert_eq!(f.params[1].ty.slot, Some(TestInput(2)));
    assert_eq!(f.ret.slot, Some(TestOutput(3)));
    assert_eq!(type_key(&f.ret.ty), "source_crate :: Key");
    assert_eq!(f.path.to_token_stream().to_string(), "source_crate :: f");
}
#[test]
fn rejects_duplicates_and_unknown_types() {
    assert!(parse("pub type Key=String; pub fn Key(){}").is_err());
    assert!(parse("pub fn f(x:Undeclared){}").is_err());
}
#[test]
fn refuses_unsupported_syntax_without_dropping_it() {
    for source in [
        "pub async fn f(){}",
        "pub fn f<T>(x:T){}",
        "pub fn f(x:Option<Unknown>){}",
        "pub struct X{field:String}",
        "fn f(){}",
        "pub enum E{A(i32)}",
    ] {
        assert!(parse(source).is_err(), "{source}");
    }
}
#[test]
fn mapping_visits_only_boundary_slots_and_preserves_source_structure() {
    let flat =
        parse("pub struct Payload{pub data:Vec<u8>} pub fn f(x:Payload)->Payload{}").unwrap();
    let guards = flat.guards.clone();
    let mut inputs = Vec::new();
    let mut outputs = Vec::new();
    let mapped = flat
        .map_slots(
            |ty, _| {
                inputs.push(type_key(ty));
                Ok(Some(TestInput(7)))
            },
            |ty, _| {
                outputs.push(type_key(ty));
                Ok(Some(TestOutput(42)))
            },
        )
        .unwrap();
    assert_eq!(inputs, ["source_crate :: Payload"]);
    assert_eq!(outputs, ["source_crate :: Payload"]);
    let field = &mapped.records.values().next().unwrap().fields[0];
    assert_eq!(field.name, "data");
    assert_eq!(type_key(&field.ty), "Vec < u8 >");
    let f = mapped.function("f").unwrap();
    assert_eq!(f.params[0].ty.slot, Some(TestInput(7)));
    assert_eq!(f.ret.slot, Some(TestOutput(42)));
    assert_eq!(f.location.crate_name.as_deref(), Some("source-crate"));
    assert_eq!(mapped.guards, guards);
}

#[test]
fn selected_sources_ignore_unbound_shapes_but_validate_reachable_ones() {
    let source = "pub type Handle = hidden::Private; pub enum Unsupported { A(i32) } pub fn make() -> Handle {} pub fn bad(x: Unsupported) {}";
    let items = || {
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default()))
    };
    let flat = Flat::<TestSchema>::parse_selected(items(), &["make".into()], &[]).unwrap();
    assert_eq!(flat.functions.len(), 1);
    assert!(flat.types.contains_key("Handle"));
    assert!(!flat.types.contains_key("Unsupported"));
    assert!(
        Flat::<TestSchema>::parse_selected(items(), &["bad".into()], &[])
            .unwrap_err()
            .contains("fieldless enum")
    );
}

#[test]
fn selected_record_fields_retain_transitive_source_types() {
    let items = syn::parse_file("pub type Key = hidden::Key; pub struct Record { pub key: Key } pub fn read(r: &Record) -> String {} pub fn unrelated(x: Option<u64>) {}").unwrap().items;
    let flat = Flat::<TestSchema>::parse_selected(
        items.into_iter().map(|i| (i, Default::default())),
        &["read".into()],
        &[],
    )
    .unwrap();
    assert!(flat.types.contains_key("Key"));
    assert!(flat.types.contains_key("Record"));
    assert!(!flat.functions.contains_key("unrelated"));
}

#[test]
fn primitives_keep_their_rust_syntax_without_accepting_lookalikes() {
    let f = parse("pub fn f(x: ::core::primitive::i64) -> std::primitive::char {} ").unwrap();
    assert_eq!(
        Primitive::from_type(&f.function("f").unwrap().params[0].ty.ty),
        Some(Primitive::I64)
    );
    assert_eq!(
        Primitive::from_type(&f.function("f").unwrap().ret.ty),
        Some(Primitive::Char)
    );
    assert!(parse("pub fn f(x: unrelated::i64) {} ").is_err());
}

#[test]
fn named_object_size_is_checked_by_rust_layout() {
    let dir = tempfile::tempdir().unwrap();
    for (source, valid) in [
        ("pub struct Empty {}", false),
        ("pub struct Empty { pub nothing: () }", false),
        ("pub type Empty = ();", false),
        ("pub type Empty = [u8; 0];", false),
        (
            "mod hidden { pub struct Token; } pub type Empty = hidden::Token;",
            false,
        ),
        ("pub struct Object { pub byte: u8, pub nothing: () }", true),
        ("pub type Object = std::rc::Rc<std::cell::Cell<u64>>;", true),
        ("pub fn no_result() {}", true),
    ] {
        let items = syn::parse_file(source).unwrap().items;
        let flat = Flat::<TestSchema>::parse(items.into_iter().filter_map(|i| {
            (!matches!(i, syn::Item::Mod(_))).then_some((
                i,
                SourceLocation {
                    crate_name: Some("source_crate".into()),
                    ..Default::default()
                },
            ))
        }))
        .unwrap()
        .map_slots(|_, slot| Ok(slot), |_, slot| Ok(slot))
        .unwrap();
        let guards = flat.guards.iter().map(ToTokens::to_token_stream);
        let file = dir.path().join("size_check.rs");
        std::fs::write(
            &file,
            format!(
                "pub mod source_crate {{ {source} }} {}",
                quote::quote!(#(#guards)*)
            ),
        )
        .unwrap();
        let output = std::process::Command::new("rustc")
            .args([
                "--edition=2021",
                "--crate-type=lib",
                "--emit=metadata",
                "--out-dir",
            ])
            .arg(dir.path())
            .arg(&file)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.success(), valid, "{source}: {stderr}");
        if !valid {
            assert!(
                stderr.contains("flat object `Empty` must have nonzero size"),
                "{stderr}"
            );
        }
    }
}

#[test]
fn options_preserve_payload_types_and_reject_malformed_arguments() {
    let flat=parse("pub struct Item {pub value:i64} pub fn f(x:Option<&Item>)->core::option::Option<std::option::Option<Item>> {}").unwrap();
    let f = flat.function("f").unwrap();
    assert!(type_key(option_inner(&f.params[0].ty.ty).unwrap()).contains("Item"));
    assert!(option_inner(option_inner(&f.ret.ty).unwrap()).is_some());
    for source in [
        "pub fn f(x:Option<i64,i64>){}",
        "pub fn f(x:Option<3>){}",
        "pub fn f(x:Something::Option<i64>){}",
    ] {
        assert!(parse(source).is_err());
    }
}

#[test]
fn fieldless_enum_source_metadata_survives_selection_and_slot_mapping() {
    let flat = parse("pub enum Code { A = -3, B, C = 8 } pub struct Record { pub code: Option<Code> } pub fn echo(x: Record) -> Record {}").unwrap();
    let source = &flat.enums["source_crate :: Code"];
    assert_eq!(
        source
            .variants
            .iter()
            .map(|v| v.name.to_string())
            .collect::<Vec<_>>(),
        ["A", "B", "C"]
    );
    assert!(source.variants[1].discriminant.is_none());
    let mapped = flat
        .map_slots(
            |_, _| Ok(Some(TestInput(0))),
            |_, _| Ok(Some(TestOutput(0))),
        )
        .unwrap();
    assert_eq!(mapped.enums.len(), 1);
    assert_eq!(mapped.enums["source_crate :: Code"].variants.len(), 3);
    for bad in [
        "enum E { A }",
        "pub enum E<T> { A }",
        "pub enum E {}",
        "pub enum E { A {} }",
        "pub enum E { #[cfg(test)] A, B }",
        "#[non_exhaustive] pub enum E { A, B }",
        "#[cfg(test)] pub enum E { A, B }",
        "#[cfg_attr(test, non_exhaustive)] pub enum E { A, B }",
    ] {
        assert!(parse(bad).is_err(), "{bad}");
    }
}

#[test]
fn result_types_are_qualified_recursively_and_standard_spellings_normalize() {
    let flat = parse("pub type Error = String; pub type Value = String; pub fn a()->Result<Option<Value>,Error>{} pub fn b()->::std::result::Result<Option<Value>,Error>{} pub fn c()->core::result::Result<Option<Value>,Error>{}").unwrap();
    let ty = &flat.function("a").unwrap().ret.ty;
    assert!(same_type(ty, &flat.function("b").unwrap().ret.ty));
    assert!(same_type(ty, &flat.function("c").unwrap().ret.ty));
    let (ok, err) = result_parts(ty).unwrap();
    assert_eq!(type_key(err), "source_crate :: Error");
    assert_eq!(type_key(option_inner(ok).unwrap()), "source_crate :: Value");
    for source in [
        "pub fn f()->Result<i32,Unknown>{}",
        "pub fn f()->Result<i32>{}",
        "pub fn f()->custom::Result<i32,String>{}",
        "pub fn f()->Result<i32,Box<dyn Error>>{}",
    ] {
        assert!(parse(source).is_err(), "{source}");
    }
    assert!(result_parts(&syn::parse_quote!(custom::Result<i32,String>)).is_none());
    assert!(result_parts(&syn::parse_quote!(Result<i32, 3>)).is_none());
}

#[test]
fn declared_opaque_type_named_result_keeps_its_existing_meaning() {
    let flat = parse("pub type Result=String;pub fn f(value:Result)->Result{}").unwrap();
    let f = flat.function("f").unwrap();
    assert_eq!(type_key(&f.ret.ty), "source_crate :: Result");
    assert!(same_type(&f.params[0].ty.ty, &f.ret.ty));
    assert!(result_parts(&f.ret.ty).is_none());
}

#[test]
fn filling_fixed_schema_slots_preserves_metadata_on_each_source_owner() {
    // The schema marker itself deliberately implements neither Clone nor Debug.
    struct OwnerSchema;
    impl Schema for OwnerSchema {
        type Input = u8;
        type Output = u16;
        type Module = String;
        type Function = String;
        type Type = String;
        type Field = String;
        type Variant = String;
    }
    let items =
        syn::parse_file("pub struct R { pub x:i32 } pub enum E { A } pub fn echo(x:R)->R{}")
            .unwrap()
            .items;
    let mut flat =
        Flat::<OwnerSchema>::parse(items.into_iter().map(|i| (i, Default::default()))).unwrap();
    flat.metadata = "module".into();
    flat.function_mut("echo").unwrap().metadata = "export".into();
    flat.type_metadata_mut(&syn::parse_quote!(R))
        .unwrap()
        .clone_from(&"record".into());
    flat.records.get_mut("crate :: R").unwrap().fields[0].metadata = "property".into();
    flat.enums.get_mut("crate :: E").unwrap().variants[0].metadata = "constant".into();
    let mut prepared = flat
        .map_slots(|_, _| Ok(Some(1u8)), |_, _| Ok(Some(2u16)))
        .unwrap();
    prepared.function_mut("echo").unwrap().params[0].ty.slot = Some(3);
    let copied = prepared.clone();
    assert_eq!(copied.metadata, "module");
    assert_eq!(copied.function("echo").unwrap().metadata, "export");
    assert_eq!(
        copied
            .type_info(&syn::parse_quote!(crate::R))
            .unwrap()
            .metadata,
        "record"
    );
    assert_eq!(copied.records["crate :: R"].fields[0].metadata, "property");
    assert_eq!(copied.enums["crate :: E"].variants[0].metadata, "constant");
    assert_eq!(copied.function("echo").unwrap().params[0].ty.slot, Some(3));
    assert_eq!(copied.function("echo").unwrap().ret.slot, Some(2));
    assert!(format!("{copied:?}").contains("property"));
}

#[test]
fn callback_parameters_qualify_arguments_without_admitting_other_impl_traits() {
    let flat = parse("pub type Item=String; pub fn f(cb: impl Fn(&Item, Option<Item>) -> () + Send + Sync + 'static) {}").unwrap();
    let callback = callback_signature(&flat.function("f").unwrap().params[0].ty.ty)
        .unwrap()
        .unwrap();
    assert_eq!(type_key(&callback.arguments[0]), "& source_crate :: Item");
    assert_eq!(
        type_key(&callback.arguments[1]),
        "Option < source_crate :: Item >"
    );
    assert!(flat.resolve_type(&syn::parse_quote!(impl Fn())).is_err());
    for source in [
        "pub fn f(cb: impl FnMut()) {}",
        "pub fn f(cb: impl FnOnce()) {}",
        "pub fn f(cb: impl Fn() -> i64) {}",
        "pub fn f(cb: impl for<'a> Fn(&'a str)) {}",
        "pub fn f(cb: impl std::ops::Fn() + other::Send) {}",
        "pub fn f(cb: impl Iterator<Item=i64>) {}",
        "pub fn f(cb: Option<impl Fn()>) {}",
        "pub fn f() -> impl Fn() {}",
        "pub struct Record { pub callback: impl Fn() }",
        "pub fn f(cb: impl Fn(Option<impl Fn()>)) {}",
    ] {
        assert!(parse(source).is_err(), "{source}");
    }
    assert!(parse("pub fn f(cb: impl std::ops::Fn() + core::marker::Send) {}").is_ok());
}
