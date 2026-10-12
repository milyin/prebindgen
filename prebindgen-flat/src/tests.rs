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
        "pub fn f(x:&mut String){}",
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
