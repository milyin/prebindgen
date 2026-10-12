use super::*;
fn parse(text: &str) -> Result<Flat<()>> {
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
    let mut flat = flat.map_slots(|_, ()| Ok(0u8)).unwrap();
    let f = flat.function_mut("f").unwrap();
    f.params[0].ty.slot = 1;
    f.params[1].ty.slot = 2;
    f.ret.slot = 3;
    assert_eq!(f.params[0].ty.slot, 1);
    assert_eq!(f.params[1].ty.slot, 2);
    assert_eq!(f.ret.slot, 3);
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
        "pub fn f(x:Option<String>){}",
        "pub struct X{field:String}",
        "fn f(){}",
        "pub enum E{A}",
    ] {
        assert!(parse(source).is_err(), "{source}");
    }
}
#[test]
fn field_syntax_and_slot_survive_mapping() {
    let flat = parse("pub struct Payload{pub data:Vec<u8>} pub fn f(x:Payload){}").unwrap();
    let mapped = flat.map_slots(|ty, ()| Ok(type_key(ty))).unwrap();
    let field = &mapped.records.values().next().unwrap().fields[0];
    assert_eq!(field.ty.slot, "Vec < u8 >");
    assert_eq!(type_key(&field.ty.ty), field.ty.slot);
}

#[test]
fn selected_sources_ignore_unbound_shapes_but_validate_reachable_ones() {
    let source = "pub type Handle = hidden::Private; pub enum Unsupported { A } pub fn make() -> Handle {} pub fn bad(x: Unsupported) {}";
    let items = || {
        syn::parse_file(source)
            .unwrap()
            .items
            .into_iter()
            .map(|i| (i, Default::default()))
    };
    let flat = Flat::<()>::parse_selected(items(), &["make".into()], &[]).unwrap();
    assert_eq!(flat.functions.len(), 1);
    assert!(flat.types.contains_key("Handle"));
    assert!(!flat.types.contains_key("Unsupported"));
    assert!(Flat::<()>::parse_selected(items(), &["bad".into()], &[])
        .unwrap_err()
        .contains("unsupported flat item"));
}

#[test]
fn selected_record_fields_retain_transitive_source_types() {
    let items = syn::parse_file("pub type Key = hidden::Key; pub struct Record { pub key: Key } pub fn read(r: &Record) -> String {} pub fn unrelated(x: Option<u64>) {}").unwrap().items;
    let flat = Flat::<()>::parse_selected(
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
        let flat = Flat::<()>::parse(items.into_iter().filter_map(|i| {
            (!matches!(i, syn::Item::Mod(_))).then_some((
                i,
                SourceLocation {
                    crate_name: Some("source_crate".into()),
                    ..Default::default()
                },
            ))
        }))
        .unwrap()
        .map_slots(|_, ()| Ok(()))
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
