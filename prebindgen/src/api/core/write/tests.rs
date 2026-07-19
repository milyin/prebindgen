use std::{
    collections::HashSet,
    time::{SystemTime, UNIX_EPOCH},
};

use proc_macro2::TokenStream;
use quote::ToTokens;

use super::*;
use crate::SourceLocation;

struct IdentityExt;

impl Prebindgen for IdentityExt {
    type Metadata = ();

    fn declared_functions(&self) -> HashSet<syn::Ident> {
        [syn::parse_quote!(a_fn), syn::parse_quote!(b_fn)]
            .into_iter()
            .collect()
    }

    fn declared_types(&self) -> HashSet<TypeKey> {
        ["AEnum", "AStruct", "BEnum", "BStruct"]
            .into_iter()
            .map(|s| TypeKey::parse(s).expect("test type"))
            .collect()
    }

    fn on_function(&self, f: &syn::ItemFn, _registry: &Registry<Self::Metadata>) -> TokenStream {
        f.to_token_stream()
    }

    fn on_struct(&self, s: &syn::ItemStruct, _registry: &Registry<Self::Metadata>) -> TokenStream {
        s.to_token_stream()
    }

    fn on_enum(&self, e: &syn::ItemEnum, _registry: &Registry<Self::Metadata>) -> TokenStream {
        e.to_token_stream()
    }

    fn on_input_type(
        &self,
        _ty: &syn::Type,
        _registry: &Registry<Self::Metadata>,
    ) -> Option<crate::api::core::prebindgen::ConverterImpl<Self::Metadata>> {
        None
    }

    fn on_output_type(
        &self,
        _ty: &syn::Type,
        _registry: &Registry<Self::Metadata>,
    ) -> Option<crate::api::core::prebindgen::ConverterImpl<Self::Metadata>> {
        None
    }
}

#[test]
fn dedup_and_sort() {
    let mut reg: Registry<()> = Registry::default();
    let key_a = TypeKey::parse("u64").expect("test type");
    let key_b = TypeKey::parse("Sample").expect("test type");
    let wire: syn::Type = syn::parse_quote!(i64);
    let wire2: syn::Type = syn::parse_quote!(*const u8);

    reg.input_types.insert(
        key_a.clone(),
        Some(TypeEntry {
            destination: wire.clone(),
            function: syn::parse_quote!(
                fn handle_to_u64_aaaa(v: i64) -> u64 {
                    v as u64
                }
            ),
            pre_stages: vec![],
            subs: vec![],
            required: true,
            niches: crate::api::core::niches::Niches::empty(),
            metadata: (),
        }),
    );
    reg.input_types.insert(
        key_b.clone(),
        Some(TypeEntry {
            destination: wire2.clone(),
            function: syn::parse_quote!(
                fn Ptr_to_Sample_bbbb(v: *const u8) -> Sample {
                    decode_sample(v)
                }
            ),
            pre_stages: vec![],
            subs: vec![],
            required: true,
            niches: crate::api::core::niches::Niches::empty(),
            metadata: (),
        }),
    );

    let items = collect_converter_items(&reg);
    assert_eq!(items.len(), 2);
    // Sorted ASCII: "Ptr_to_Sample_bbbb" < "handle_to_u64_aaaa"
    // (uppercase P < lowercase h).
    assert_eq!(items[0].0.to_string(), "Ptr_to_Sample_bbbb");
    assert_eq!(items[1].0.to_string(), "handle_to_u64_aaaa");
}

#[test]
fn write_rust_sorts_declared_items_by_ident() {
    let mut reg: Registry<()> = Registry::default();
    let loc = SourceLocation::default();

    reg.functions.insert(
        syn::parse_quote!(b_fn),
        (
            syn::parse_quote!(
                fn b_fn() {}
            ),
            loc.clone(),
        ),
    );
    reg.functions.insert(
        syn::parse_quote!(a_fn),
        (
            syn::parse_quote!(
                fn a_fn() {}
            ),
            loc.clone(),
        ),
    );
    reg.structs.insert(
        syn::parse_quote!(BStruct),
        (
            syn::parse_quote!(
                pub struct BStruct;
            ),
            loc.clone(),
        ),
    );
    reg.structs.insert(
        syn::parse_quote!(AStruct),
        (
            syn::parse_quote!(
                pub struct AStruct;
            ),
            loc.clone(),
        ),
    );
    reg.enums.insert(
        syn::parse_quote!(BEnum),
        (
            syn::parse_quote!(
                pub enum BEnum {
                    B,
                }
            ),
            loc.clone(),
        ),
    );
    reg.enums.insert(
        syn::parse_quote!(AEnum),
        (
            syn::parse_quote!(
                pub enum AEnum {
                    A,
                }
            ),
            loc.clone(),
        ),
    );
    reg.consts.insert(
        syn::parse_quote!(B_CONST),
        (
            syn::parse_quote!(
                pub const B_CONST: u32 = 2;
            ),
            loc.clone(),
        ),
    );
    reg.consts.insert(
        syn::parse_quote!(A_CONST),
        (
            syn::parse_quote!(
                pub const A_CONST: u32 = 1;
            ),
            loc,
        ),
    );

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock drift")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("prebindgen-write-rust-{unique}.rs"));
    let written = write_rust(&reg, &IdentityExt, &path).expect("write_rust");
    let content = std::fs::read_to_string(&written).expect("read generated file");
    let _ = std::fs::remove_file(&written);

    assert!(
        content.find("pub const A_CONST").unwrap() < content.find("pub const B_CONST").unwrap()
    );
    assert!(content.find("pub enum AEnum").unwrap() < content.find("pub enum BEnum").unwrap());
    assert!(
        content.find("pub struct AStruct").unwrap() < content.find("pub struct BStruct").unwrap()
    );
    assert!(content.find("fn a_fn").unwrap() < content.find("fn b_fn").unwrap());
}

#[test]
fn bad_generated_tokens_report_emission_phase() {
    let err = parse_items_from_tokens("on_function", [quote::quote!(fn broken)])
        .expect_err("invalid item tokens should fail");
    assert!(
        err.to_string().contains("on_function"),
        "error should mention the adapter emission phase: {}",
        err
    );
}
