use std::time::{SystemTime, UNIX_EPOCH};

use prebindgen::SourceLocation;
use proc_macro2::TokenStream;

use super::*;
use crate::registry::{Direction, RegistryBuilder};

struct IdentityExt;

impl IdentityExt {
    fn declare_into(&self, mut reg: RegistryBuilder<()>) -> RegistryBuilder<()> {
        for f in [syn::parse_quote!(a_fn), syn::parse_quote!(b_fn)] {
            reg = reg.export(&f);
        }
        for t in ["AEnum", "AStruct", "BEnum", "BStruct"] {
            reg = reg.export_type(crate::test_util::declared_origin(
                syn::parse_str(t).expect("test type"),
            ));
        }
        reg
    }
}

impl Prebindgen for IdentityExt {
    type Metadata = ();

    fn on_function(
        &self,
        f: &prebindgen_flat::flat::Function,
        _registry: &Registry<Self::Metadata>,
        emit: &prebindgen_flat::Emit,
    ) -> TokenStream {
        emit.verbatim_fn(f)
    }

    fn on_struct(
        &self,
        s: &prebindgen_flat::flat::Struct,
        _registry: &Registry<Self::Metadata>,
        emit: &prebindgen_flat::Emit,
    ) -> TokenStream {
        emit.verbatim_struct(s)
    }

    fn on_variant(
        &self,
        v: &prebindgen_flat::flat::Variant,
        _registry: &Registry<Self::Metadata>,
        emit: &prebindgen_flat::Emit,
    ) -> TokenStream {
        emit.verbatim_variant(v)
    }

    fn on_enum(
        &self,
        e: &prebindgen_flat::flat::Enum,
        _registry: &Registry<Self::Metadata>,
        emit: &prebindgen_flat::Emit,
    ) -> TokenStream {
        emit.verbatim_enum(e)
    }
}

#[test]
fn dedup_and_sort() {
    let mut reg: Registry<()> = Registry::empty();
    let ty_a: syn::Type = syn::parse_quote!(u64);
    let ty_b: syn::Type = syn::parse_quote!(Sample);
    let wire: syn::Type = syn::parse_quote!(i64);
    let wire2: syn::Type = syn::parse_quote!(*const u8);

    reg.insert_crossing(
        Direction::Input,
        &ty_a,
        true,
        Some(TypeEntry {
            destination: wire.clone(),
            function: syn::parse_quote!(
                fn handle_to_u64_aaaa(v: i64) -> u64 {
                    v as u64
                }
            ),
            pre_stages: vec![],
            subs: vec![],
            niches: crate::niches::Niches::empty(),
            metadata: (),
        }),
    );
    reg.insert_crossing(
        Direction::Input,
        &ty_b,
        true,
        Some(TypeEntry {
            destination: wire2.clone(),
            function: syn::parse_quote!(
                fn Ptr_to_Sample_bbbb(v: *const u8) -> Sample {
                    decode_sample(v)
                }
            ),
            pre_stages: vec![],
            subs: vec![],
            niches: crate::niches::Niches::empty(),
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
    // Fed in a deliberately un-sorted order: the assertion below is that
    // emission sorts by name, and the model preserves stream order.
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::parse_quote!(
                fn b_fn() {}
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                fn a_fn() {}
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub struct BStruct;
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub struct AStruct;
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub enum BEnum {
                    B,
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub enum AEnum {
                    A,
                }
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub const B_CONST: u32 = 2;
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub const A_CONST: u32 = 1;
            ),
            loc,
        ),
    ];
    let reg: Registry<()> = IdentityExt
        .declare_into(crate::test_util::reg_from_items(items).expect("index"))
        .scanned()
        .expect("scan");

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

/// An adapter with a const mechanism gates **named** consts and cannot gate
/// guards — pinned at the emission site, not just in the registry.
///
/// `a_guard_never_reaches_the_const_surface` proves the maps are separate, but it
/// never calls `write_rust`. This is what would catch a change that keeps
/// `Registry::guards` populated and then forgets to emit them, or re-gates them
/// on the way out.
#[test]
fn guards_emit_ungated_and_in_stream_order() {
    /// Declares a const mechanism and declares nothing through it, so
    /// `KEPT_OUT` must not emit.
    struct ConstGatingExt;

    trait ResolveGating {
        fn resolve_gating(self, ext: ConstGatingExt)
            -> Result<Registry<()>, crate::WriteRustError>;
    }
    impl ResolveGating for RegistryBuilder<()> {
        fn resolve_gating(
            self,
            ext: ConstGatingExt,
        ) -> Result<Registry<()>, crate::WriteRustError> {
            let registry = self.declares_consts().build()?;
            let _ = &ext;
            Ok(registry)
        }
    }

    impl Prebindgen for ConstGatingExt {
        type Metadata = ();

        fn on_function(
            &self,
            f: &prebindgen_flat::flat::Function,
            _r: &Registry<()>,
            _emit: &prebindgen_flat::Emit,
        ) -> TokenStream {
            f.origin.spell()
        }
        fn on_struct(
            &self,
            s: &prebindgen_flat::flat::Struct,
            _r: &Registry<()>,
            _emit: &prebindgen_flat::Emit,
        ) -> TokenStream {
            s.origin.spell()
        }
        fn on_variant(
            &self,
            v: &prebindgen_flat::flat::Variant,
            _r: &Registry<()>,
            _emit: &prebindgen_flat::Emit,
        ) -> TokenStream {
            v.origin.spell()
        }
        fn on_enum(
            &self,
            e: &prebindgen_flat::flat::Enum,
            _r: &Registry<()>,
            _emit: &prebindgen_flat::Emit,
        ) -> TokenStream {
            e.origin.spell()
        }
    }

    let loc = SourceLocation::default();
    // Two distinguishable guards, straddling the named const, so the assertion
    // below pins order rather than merely presence.
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::parse_quote!(
                const _: () = {
                    first_check();
                };
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                pub const KEPT_OUT: u64 = 7;
            ),
            loc.clone(),
        ),
        (
            syn::parse_quote!(
                const _: () = {
                    second_check();
                };
            ),
            loc.clone(),
        ),
    ];
    let registry: RegistryBuilder<()> = crate::test_util::reg_from_items(items).expect("index");
    assert_eq!(registry.flat().guards().count(), 2);

    let dir = crate::test_util::unique_test_dir("write_guards");
    std::fs::create_dir_all(&dir).unwrap();
    let registry = registry.resolve_gating(ConstGatingExt).expect("resolve");
    let path = crate::write::write_rust(&registry, &ConstGatingExt, dir.join("gen.rs"))
        .expect("write_rust");
    let src = std::fs::read_to_string(&path).unwrap();

    // The named const is gated out; both guards emit regardless.
    assert!(
        !src.contains("KEPT_OUT"),
        "declared_consts is empty:\n{src}"
    );
    let first = src
        .find("first_check")
        .unwrap_or_else(|| panic!("guard 1 missing:\n{src}"));
    let second = src
        .find("second_check")
        .unwrap_or_else(|| panic!("guard 2 missing:\n{src}"));
    assert!(first < second, "guards must keep stream order:\n{src}");
}
