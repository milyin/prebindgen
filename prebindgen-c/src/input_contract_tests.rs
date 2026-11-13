//! Public preparation/emission regressions for independently prepared leaves.
use super::*;

#[test]
fn every_mixed_raw_and_declared_input_combination_generates() {
    let cases: &[(&str, &str, &[usize])] = &[
        ("&Item", "Item", &[]),
        ("&Item", "Item", &[0]),
        ("&Item", "Item", &[1]),
        ("&Item", "Item", &[0, 1]),
        ("&Item", "&Item", &[0]),
        ("&Item", "i64", &[0]),
        ("&Item", "Packet", &[]),
        ("&Item", "Packet", &[0]),
    ];
    for (first, second, raw) in cases {
        let source = format!("pub struct Item{{pub value:i64}} pub struct Packet{{pub id:u32}} pub fn mixed(first:{first},second:{second}){{}}");
        let items = syn::parse_file(&source).unwrap().items;
        let flat =
            CBindings::parse(items.into_iter().map(|item| (item, Default::default()))).unwrap();
        let mut model = Cbindgen::new(flat)
            .opaque_ptr("item", syn::parse_quote!(Item))
            .opaque_ptr("packet", syn::parse_quote!(Packet))
            .function(prebindgen_declarations::fun!(mixed))
            .panic()
            .prepare()
            .unwrap();
        for &index in *raw {
            let ty = model.function("mixed").unwrap().params[index].ty.ty.clone();
            let plan = CCodec::raw()
                .input_leaf(InputLeaf::new(
                    &model,
                    &ty,
                    &(),
                    format_ident!("raw_{index}"),
                    format_ident!("raw_value_{index}"),
                    &Default::default(),
                ))
                .unwrap();
            model.function_mut("mixed").unwrap().params[index].ty.slot = Some(plan);
        }
        let generated = generate(&model).unwrap();
        syn::parse_file(&generated.rust).unwrap();
        assert!(!generated.rust.contains("__prebindgen_input_transfer"));
    }
}
