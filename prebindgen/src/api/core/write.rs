//! Rust file emission for the resolved `Registry`.
//!
//! `write_rust` collects every resolved input/output converter (each entry
//! already carries its full `ItemFn`), every per-item `on_<kind>` output,
//! and every anonymous const; concatenates them; and hands them to
//! `Destination::write` (which does prettyplease formatting and
//! resolves the path against `OUT_DIR`).
//!
//! This module is `pub`, so **every `pub` item in it is public API of the
//! crate**. That is meant to be exactly two — [`write_rust`] and
//! [`WriteError`] — which is what an out-of-crate adapter calls to emit its
//! generated file. Anything else added here stays private unless publishing it
//! is a deliberate decision.

use std::{
    collections::BTreeMap,
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
///
/// Binding validation is NOT here — it runs once in [`Registry::finish`]
/// (see [`Prebindgen::validate_resolved`]), so an invalid binding fails
/// before a built generator exists and never reaches a writer.
#[derive(Debug)]
pub enum WriteError {
    /// A `TokenStream` produced by an `on_*` trait method failed to parse
    /// as `syn::Item`s. Indicates a codegen bug in the adapter.
    BadTokens {
        phase: &'static str,
        source: syn::Error,
    },
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
    // Validation already ran ONCE in the generator's `build` — a built generator
    // (the only source of a resolved registry) is valid by construction, so
    // this writer is a pure emission.
    // The capability, minted here and nowhere else in this function's reach.
    // Every callback below is handed a borrow; nothing else in the pipeline is.
    // See `core::emit` for what that buys and what it deliberately does not.
    let emit = crate::api::core::emit::Emit::new();
    let mut items: Vec<syn::Item> = Vec::new();

    // 0. Adapter prerequisites — runtime-support items (helper structs,
    //    type aliases) the converter bodies depend on. Emitted first so
    //    everything below can reference them.
    items.extend(ext.prerequisites(registry, &emit));

    // 1. Auto-generated converter wrappers (sorted by ident, deduped).
    for (_, item_fn) in collect_converter_items(registry) {
        items.push(syn::Item::Fn(item_fn));
    }

    // 2. Per-item Rust output from the adapter — only for items the adapter
    //    explicitly declared. Undeclared items were already announced
    //    via `cargo:warning=` by the generator's own unclaimed-item report.
    let declared = registry.declared();
    let declared_fns = &declared.functions;
    let declared_types = &declared.types;
    let flat = registry.flat();
    items.extend(parse_items_from_tokens(
        "on_function",
        sorted_by_name(flat.functions().map(|f| (&f.name, f)))
            .into_iter()
            .filter(|(ident, _)| declared_fns.contains(*ident))
            .map(|(_, item)| ext.on_function(item, registry, &emit)),
    )?);
    items.extend(parse_items_from_tokens(
        "on_struct",
        sorted_by_name(flat.types().filter_map(|t| match t {
            crate::api::core::flat::Type::Struct(s) => Some((&s.name, s)),
            _ => None,
        }))
        .into_iter()
        .filter(|(ident, _)| declared_types.contains_key(&TypeKey::from_ident(ident)))
        .map(|(_, item)| ext.on_struct(item, registry, &emit)),
    )?);
    // Both enum shapes emit through `on_enum` and sort together: they were one
    // map here before they were two elements. They still SORT together — the
    // emission order is one sequence — but they dispatch to their own methods
    // now, because handing an adapter a `Type` it has to re-match is worse than
    // handing it the element the model already decided on.
    items.extend(parse_items_from_tokens(
        "on_enum",
        sorted_by_name(flat.types().filter_map(|t| match t {
            crate::api::core::flat::Type::Variant(v) => Some((&v.name, t)),
            crate::api::core::flat::Type::Enum(e) => Some((&e.name, t)),
            _ => None,
        }))
        .into_iter()
        .filter(|(ident, _)| declared_types.contains_key(&TypeKey::from_ident(ident)))
        .map(|(_, t)| match t {
            crate::api::core::flat::Type::Variant(v) => ext.on_variant(v, registry, &emit),
            crate::api::core::flat::Type::Enum(e) => ext.on_enum(e, registry, &emit),
            _ => unreachable!("filtered to the two enum shapes above"),
        }),
    )?);
    // Consts: an adapter WITH a const declaration mechanism
    // (`declared_consts() == Some(set)`) emits declared consts only,
    // symmetric with functions; an adapter without one (`None`) gets every
    // const passed through verbatim via the default `on_const`. Prebindgen's
    // own injected feature guards are not consts at all — see the guards loop.
    let declared_consts = &declared.consts;
    items.extend(parse_items_from_tokens(
        "on_const",
        sorted_by_name(flat.constants().map(|c| (&c.name, c)))
            .into_iter()
            .filter(|(ident, _)| {
                declared_consts
                    .as_ref()
                    .is_none_or(|set| set.contains(*ident))
            })
            .map(|(_, item)| ext.on_const(item, registry, &emit)),
    )?);

    // 3. Anonymous consts, verbatim. Last, and in stream order. Ungated on
    //    purpose: with no name there is nothing for an adapter to declare, so
    //    the const gate above cannot apply to them.
    for guard in flat.guards() {
        items.push(syn::Item::Const(emit.guard(guard)));
    }

    // 4. Cross-cutting post-process pass. Adapters use this to qualify
    //    bare type references etc. — see Prebindgen::post_process_item.
    for item in &mut items {
        ext.post_process_item(item, registry, &emit);
    }

    let dest: Destination = items.into_iter().collect();
    Ok(dest.write(out_path))
}

/// Walk both type tables, dedupe each entry's stored `function` AND each
/// of its [`crate::api::core::prebindgen::Stage`] functions by name, sort
/// for determinism. Names are read directly off `function.sig.ident` —
/// the adapter owns the naming.
///
/// Private: an internal step of [`write_rust`], not part of the
/// adapter-facing surface this module exposes.
fn collect_converter_items<M>(registry: &Registry<M>) -> Vec<(syn::Ident, syn::ItemFn)> {
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
    table: &std::collections::HashMap<TypeKey, crate::api::core::registry::TypeCell<M>>,
    mut f: F,
) {
    let mut keys: Vec<&TypeKey> = table.keys().collect();
    keys.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    for key in keys {
        if let Some(entry) = table.get(key).and_then(|c| c.entry.as_ref()) {
            f(key, entry);
        }
    }
}

/// Name-sorted, because emission order is part of the generated file and the
/// model is in source order. Was `sorted_items_by_ident` over the registry's
/// maps; same ordering, read from the one index.
fn sorted_by_name<'a, T>(
    items: impl Iterator<Item = (&'a syn::Ident, &'a T)>,
) -> Vec<(&'a syn::Ident, &'a T)>
where
    T: 'a,
{
    let mut items: Vec<(&syn::Ident, &T)> = items.collect();
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
