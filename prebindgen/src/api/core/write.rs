//! Rust file emission for the resolved `Registry`.
//!
//! `write_rust` collects every resolved input/output converter (each entry
//! already carries its full `ItemFn`), every per-item `on_<kind>` output,
//! and every passthrough item; concatenates them; and hands them to
//! `crate::collect::Destination::write` (which does prettyplease
//! formatting and resolves the path against `OUT_DIR`).

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

use crate::collect::Destination;
use proc_macro2::TokenStream;

use crate::api::core::prebindgen::Prebindgen;
use crate::api::core::registry::{Registry, TypeEntry, TypeKey};

/// Errors surfaced by the file-emission phase.
#[derive(Debug)]
pub enum WriteError {
    /// A `TokenStream` produced by an `on_*` trait method failed to parse
    /// as `syn::Item`s. Indicates a codegen bug in the adapter.
    BadTokens(syn::Error),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::BadTokens(e) => write!(f, "generated tokens did not parse: {}", e),
        }
    }
}

impl std::error::Error for WriteError {}

/// Emit the resolved registry to a Rust file.
///
/// `out_path` may be relative (resolved against `OUT_DIR` by prebindgen) or
/// absolute. Returns the path actually written.
pub fn write_rust<P: AsRef<Path>, E: Prebindgen>(
    registry: &Registry<E::Metadata>,
    ext: &E,
    out_path: P,
) -> Result<PathBuf, WriteError> {
    let mut items: Vec<syn::Item> = Vec::new();

    // 0. Adapter prerequisites — runtime-support items (helper structs,
    //    type aliases) the converter bodies depend on. Emitted first so
    //    everything below can reference them.
    items.extend(ext.prerequisites(registry));

    // 1. Auto-generated converter wrappers (sorted by ident, deduped).
    for (_, item_fn) in collect_converter_items(registry) {
        items.push(syn::Item::Fn(item_fn));
    }

    // 2. Per-item Rust output from the adapter — only for items the adapter
    //    explicitly declared. Undeclared items were already announced
    //    via `cargo:warning=` in `Registry::scan_declared`.
    let declared_fns = ext.declared_functions();
    let declared_types = ext.declared_types();
    items.extend(parse_items_from_tokens(
        sorted_items_by_ident(&registry.functions)
            .into_iter()
            .filter(|(ident, _)| declared_fns.contains(*ident))
            .map(|(_, (item, _))| ext.on_function(item, registry)),
    )?);
    items.extend(parse_items_from_tokens(
        sorted_items_by_ident(&registry.structs)
            .into_iter()
            .filter(|(ident, _)| declared_types.contains(&TypeKey::parse(&ident.to_string())))
            .map(|(_, (item, _))| ext.on_struct(item, registry)),
    )?);
    items.extend(parse_items_from_tokens(
        sorted_items_by_ident(&registry.enums)
            .into_iter()
            .filter(|(ident, _)| declared_types.contains(&TypeKey::parse(&ident.to_string())))
            .map(|(_, (item, _))| ext.on_enum(item, registry)),
    )?);
    // Consts: always emit verbatim — declaration mechanism for consts
    // is future work (see plan).
    items.extend(parse_items_from_tokens(
        sorted_items_by_ident(&registry.consts)
            .into_iter()
            .map(|(_, (item, _))| ext.on_const(item, registry)),
    )?);

    // 3. Passthrough items verbatim.
    for (item, _) in &registry.passthrough {
        items.push(item.clone());
    }

    // 4. Cross-cutting post-process pass. Adapters use this to qualify
    //    bare type references etc. — see Prebindgen::post_process_item.
    for item in &mut items {
        ext.post_process_item(item);
    }

    let dest: Destination = items.into_iter().collect();
    Ok(dest.write(out_path))
}

/// Walk both type tables, dedupe each entry's stored `function` AND each
/// of its [`crate::api::core::prebindgen::Stage`] functions by name, sort
/// for determinism. Names are read directly off `function.sig.ident` —
/// the adapter owns the naming.
pub fn collect_converter_items<M>(registry: &Registry<M>) -> Vec<(syn::Ident, syn::ItemFn)> {
    let mut by_name: BTreeMap<String, (syn::Ident, syn::ItemFn)> = BTreeMap::new();
    let mut collect = |entry: &TypeEntry<M>| {
        let name = entry.function.sig.ident.clone();
        by_name
            .entry(name.to_string())
            .or_insert_with(|| (name, entry.function.clone()));
        for stage in &entry.pre_stages {
            let sname = stage.function.sig.ident.clone();
            by_name
                .entry(sname.to_string())
                .or_insert_with(|| (sname, stage.function.clone()));
        }
    };
    walk_resolved(&registry.input_types, |_, entry| collect(entry));
    walk_resolved(&registry.output_types, |_, entry| collect(entry));
    by_name.into_values().collect()
}

fn walk_resolved<M, F: FnMut(&TypeKey, &TypeEntry<M>)>(
    table: &std::collections::HashMap<TypeKey, Option<TypeEntry<M>>>,
    mut f: F,
) {
    let mut keys: Vec<&TypeKey> = table.keys().collect();
    keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    for key in keys {
        if let Some(Some(entry)) = table.get(key) {
            f(key, entry);
        }
    }
}

fn sorted_items_by_ident<T>(map: &HashMap<syn::Ident, T>) -> Vec<(&syn::Ident, &T)> {
    let mut items: Vec<(&syn::Ident, &T)> = map.iter().collect();
    items.sort_by(|(left, _), (right, _)| left.to_string().cmp(&right.to_string()));
    items
}

/// Parse a per-item `TokenStream` (which may be empty) as a sequence of
/// `syn::Item`s. Empty token streams yield zero items.
fn parse_items_from_tokens<I: IntoIterator<Item = TokenStream>>(
    iter: I,
) -> Result<Vec<syn::Item>, WriteError> {
    let mut out = Vec::new();
    for ts in iter {
        if ts.is_empty() {
            continue;
        }
        let file: syn::File = syn::parse2(ts.clone()).map_err(WriteError::BadTokens)?;
        out.extend(file.items);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SourceLocation;
    use proc_macro2::TokenStream;
    use quote::ToTokens;
    use std::collections::HashSet;
    use std::time::{SystemTime, UNIX_EPOCH};

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
                .map(TypeKey::parse)
                .collect()
        }

        fn on_function(
            &self,
            f: &syn::ItemFn,
            _registry: &Registry<Self::Metadata>,
        ) -> TokenStream {
            f.to_token_stream()
        }

        fn on_struct(
            &self,
            s: &syn::ItemStruct,
            _registry: &Registry<Self::Metadata>,
        ) -> TokenStream {
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
        let key_a = TypeKey::parse("u64");
        let key_b = TypeKey::parse("Sample");
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
            content.find("pub struct AStruct").unwrap()
                < content.find("pub struct BStruct").unwrap()
        );
        assert!(content.find("fn a_fn").unwrap() < content.find("fn b_fn").unwrap());
    }
}
