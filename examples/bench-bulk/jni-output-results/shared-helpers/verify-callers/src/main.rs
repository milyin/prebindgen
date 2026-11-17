use quote::ToTokens;
use std::{collections::BTreeMap, fs};
use syn::{
    visit_mut::{self, VisitMut},
    Item, Stmt,
};
fn cleanup(s: &str) -> bool {
    s.starts_with("__pb_cleanup_") || s.ends_with("_release_row") || s == "release_nested"
}
struct Normalize;
impl VisitMut for Normalize {
    fn visit_block_mut(&mut self, b: &mut syn::Block) {
        b.stmts
            .retain(|s| !matches!(s,Stmt::Item(Item::Fn(f)) if cleanup(&f.sig.ident.to_string())));
        visit_mut::visit_block_mut(self, b);
    }
    fn visit_ident_mut(&mut self, i: &mut syn::Ident) {
        if cleanup(&i.to_string()) {
            *i = syn::Ident::new("__cleanup_reference", i.span());
        }
    }
}
fn read(p: &str) -> BTreeMap<String, String> {
    syn::parse_file(&fs::read_to_string(p).unwrap())
        .unwrap()
        .items
        .into_iter()
        .filter_map(|i| {
            let Item::Fn(mut f) = i else { return None };
            let name = f.sig.ident.to_string();
            if cleanup(&name) {
                return None;
            };
            Normalize.visit_item_fn_mut(&mut f);
            Some((name, f.to_token_stream().to_string()))
        })
        .collect()
}
fn main() {
    let a: Vec<_> = std::env::args().collect();
    let before = read(&a[1]);
    let after = read(&a[2]);
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    let changed: Vec<_> = before
        .iter()
        .filter(|(k, v)| after.get(*k) != Some(*v))
        .map(|(k, _)| k)
        .collect();
    println!("Compared {} functions; {} differ after removing local cleanup declarations and normalizing cleanup references",before.len(),changed.len());
    for name in &changed {
        println!("CHANGED {}", name);
    }
    assert!(changed.is_empty());
}
