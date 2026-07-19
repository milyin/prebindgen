//! Rust file emission for the resolved `Registry`.
//!
//! `write_rust` collects every resolved input/output converter (each entry
//! already carries its full `ItemFn`), every per-item `on_<kind>` output,
//! and every passthrough item; concatenates them; and hands them to
//! `Destination::write` (which does prettyplease formatting and
//! resolves the path against `OUT_DIR`).

use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream;

use crate::api::{
    collect::destination::Destination,
    core::{
        prebindgen::Prebindgen,
        registry::{Registry, TypeEntry, TypeKey},
    },
};

/// Errors surfaced by the file-emission phase.
#[derive(Debug)]
pub enum WriteError {
    /// A `TokenStream` produced by an `on_*` trait method failed to parse
    /// as `syn::Item`s. Indicates a codegen bug in the adapter.
    BadTokens {
        phase: &'static str,
        source: syn::Error,
    },
    /// The adapter's post-resolve validation
    /// ([`Prebindgen::validate_resolved`]) rejected the binding — nothing
    /// was written.
    Validation(String),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::BadTokens { phase, source } => {
                write!(
                    f,
                    "generated tokens from {} did not parse: {}",
                    phase, source
                )
            }
            WriteError::Validation(msg) => write!(f, "binding validation failed: {}", msg),
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
    // Post-resolve validation boundary: every artifact writer runs it first,
    // so an invalid binding fails cleanly before ANY file is written —
    // regardless of which artifact is written first.
    ext.validate_resolved(registry)
        .map_err(WriteError::Validation)?;

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
        "on_function",
        sorted_items_by_ident(&registry.functions)
            .into_iter()
            .filter(|(ident, _)| declared_fns.contains(*ident))
            .map(|(_, (item, _))| ext.on_function(item, registry)),
    )?);
    items.extend(parse_items_from_tokens(
        "on_struct",
        sorted_items_by_ident(&registry.structs)
            .into_iter()
            .filter(|(ident, _)| declared_types.contains(&TypeKey::from_ident(ident)))
            .map(|(_, (item, _))| ext.on_struct(item, registry)),
    )?);
    items.extend(parse_items_from_tokens(
        "on_enum",
        sorted_items_by_ident(&registry.enums)
            .into_iter()
            .filter(|(ident, _)| declared_types.contains(&TypeKey::from_ident(ident)))
            .map(|(_, (item, _))| ext.on_enum(item, registry)),
    )?);
    // Consts: an adapter WITH a const declaration mechanism
    // (`declared_consts() == Some(set)`) emits declared consts only,
    // symmetric with functions; an adapter without one (`None`) gets every
    // const passed through verbatim via the default `on_const`. Unnamed
    // consts (`const _`, e.g. the injected `konst::assertc_eq!` feature
    // guard) are infrastructure, not declarable API — they bypass the gate
    // and always emit.
    let declared_consts = ext.declared_consts();
    items.extend(parse_items_from_tokens(
        "on_const",
        sorted_items_by_ident(&registry.consts)
            .into_iter()
            .filter(|(ident, _)| {
                *ident == "_"
                    || declared_consts
                        .as_ref()
                        .is_none_or(|set| set.contains(*ident))
            })
            .map(|(_, (item, _))| ext.on_const(item, registry)),
    )?);

    // 3. Passthrough items verbatim.
    for (item, _) in &registry.passthrough {
        items.push(item.clone());
    }

    // 4. Cross-cutting post-process pass. Adapters use this to qualify
    //    bare type references etc. — see Prebindgen::post_process_item.
    for item in &mut items {
        ext.post_process_item(item, registry);
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
    items.sort_by_key(|(left, _)| left.to_string());
    items
}

/// Parse a per-item `TokenStream` (which may be empty) as a sequence of
/// `syn::Item`s. Empty token streams yield zero items.
fn parse_items_from_tokens<I: IntoIterator<Item = TokenStream>>(
    phase: &'static str,
    iter: I,
) -> Result<Vec<syn::Item>, WriteError> {
    let mut out = Vec::new();
    for ts in iter {
        if ts.is_empty() {
            continue;
        }
        let file: syn::File =
            syn::parse2(ts.clone()).map_err(|source| WriteError::BadTokens { phase, source })?;
        out.extend(file.items);
    }
    Ok(out)
}

#[cfg(test)]
mod tests;
