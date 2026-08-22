//! Rust file emission for the resolved `Registry`.
//!
//! `write_rust` takes the conversions the adapter compiled, as a slice of
//! [`RustFunction`] plans; renders them with the writer-owned [`crate::Emit`],
//! adds every per-item `on_<kind>` output and every anonymous const, and hands
//! the assembled file to `Destination::write`.
//!
//! The conversions arrive from the adapter rather than being collected from the
//! registry, which is what lets one crossing contribute more than one function
//! — or one that occupies more than a single wire value.
//!
//! This module is `pub`, so **every `pub` item in it is public API of the
//! crate**. [`RustFunction`] is the deliberately narrow late-rendering seam for
//! out-of-crate adapters; a complete `syn::ItemFn` remains a valid plan for
//! adapters migrating incrementally.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream;

use crate::{
    destination::Destination,
    prebindgen::Prebindgen,
    registry::{Registry, TypeKey},
};

/// Errors surfaced by the file-emission phase.
///
/// Binding validation is NOT here — it runs once in
/// [`RegistryBuilder::build`](crate::RegistryBuilder::build)
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

/// One planned private converter function.
///
/// Resolution may keep an adapter-specific semantic plan here instead of a
/// rendered Rust body. The writer calls this only after planning and
/// validation are complete, with the same [`crate::Emit`] capability used for
/// the rest of final Rust emission.
pub trait RustFunction {
    /// Whether this plan is reachable from the generated adapter surface.
    /// Validation-only plans may return false and remain available for diagnostics.
    fn should_emit(&self) -> bool {
        true
    }

    /// Materialize the complete private converter function.
    fn render(&self, emit: &crate::Emit) -> syn::ItemFn;
}

impl RustFunction for syn::ItemFn {
    fn render(&self, _emit: &crate::Emit) -> syn::ItemFn {
        self.clone()
    }
}

/// Emit a resolved registry whose private converters are rendered at this
/// final writing boundary.
///
/// `conversions` is what the adapter's compilation produced. It is sorted and
/// de-duplicated by function name here, so the order decides which of two
/// same-named functions wins and not where any of them lands. Handing them
/// over directly is what frees an adapter to emit a conversion the converter
/// table could not hold — several functions for one crossing, or one occupying
/// more than a single wire value.
///
/// Already-rendered [`syn::ItemFn`] values implement [`RustFunction`], so
/// adapters can migrate to semantic plans incrementally without a second API.
///
/// `out_path` may be relative (resolved against `OUT_DIR` by prebindgen) or
/// absolute. Returns the path actually written.
pub fn write_rust<P: AsRef<Path>, E: Prebindgen, C: RustFunction>(
    registry: &Registry,
    ext: &E,
    conversions: &[C],
    out_path: P,
) -> Result<PathBuf, WriteError> {
    // Validation already ran ONCE in the generator's `build` — a built generator
    // (the only source of a resolved registry) is valid by construction, so
    // this writer is a pure emission.
    // The capability, minted here and nowhere else in this function's reach.
    // Every callback below is handed a borrow; nothing else in the pipeline is.
    // See `prebindgen_flat::flat::emit` for what that buys and what it
    // deliberately does not.
    let emit = crate::Emit::new();
    let mut items: Vec<syn::Item> = Vec::new();

    // 0. Adapter prerequisites — runtime-support items (helper structs,
    //    type aliases) the converter bodies depend on. Emitted first so
    //    everything below can reference them.
    items.extend(ext.prerequisites(registry, &emit));

    // 2. Per-item Rust output from the adapter — only for items the adapter
    //    explicitly declared. Undeclared items were already announced
    //    via `cargo:warning=` by the generator's own unclaimed-item report.
    let declared = registry.declared();
    let declared_fns = &declared.functions;
    let declared_types = &declared.types;
    let flat = registry.flat();
    let mut body_items: Vec<syn::Item> = Vec::new();
    body_items.extend(parse_items_from_tokens(
        "on_function",
        sorted_by_name(flat.functions().map(|f| (&f.name, f)))
            .into_iter()
            .filter(|(ident, _)| declared_fns.contains(*ident))
            .map(|(_, item)| ext.on_function(item, registry, &emit)),
    )?);
    body_items.extend(parse_items_from_tokens(
        "on_struct",
        sorted_by_name(flat.types().filter_map(|t| match t {
            prebindgen_flat::flat::Type::Struct(s) => Some((&s.name, s)),
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
    body_items.extend(parse_items_from_tokens(
        "on_enum",
        sorted_by_name(flat.types().filter_map(|t| match t {
            prebindgen_flat::flat::Type::Variant(v) => Some((&v.name, t)),
            prebindgen_flat::flat::Type::Enum(e) => Some((&e.name, t)),
            _ => None,
        }))
        .into_iter()
        .filter(|(ident, _)| declared_types.contains_key(&TypeKey::from_ident(ident)))
        .map(|(_, t)| match t {
            prebindgen_flat::flat::Type::Variant(v) => ext.on_variant(v, registry, &emit),
            prebindgen_flat::flat::Type::Enum(e) => ext.on_enum(e, registry, &emit),
            _ => unreachable!("filtered to the two enum shapes above"),
        }),
    )?);
    // Consts: an adapter WITH a const declaration mechanism
    // (`declared_consts() == Some(set)`) emits declared consts only,
    // symmetric with functions; an adapter without one (`None`) gets every
    // const passed through verbatim via the default `on_const`. Prebindgen's
    // own injected feature guards are not consts at all — see the guards loop.
    let declared_consts = &declared.consts;
    body_items.extend(parse_items_from_tokens(
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

    // Render converters only after per-item planning has marked the late plans
    // reachable, while still placing them before adapter output in the file.
    let conversions = conversions
        .iter()
        .filter(|plan| plan.should_emit())
        .map(|plan| plan.render(&emit))
        .collect();
    for (_, item_fn) in dedup_by_name(conversions) {
        items.push(syn::Item::Fn(item_fn));
    }
    items.extend(body_items);

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

/// Sort by name and keep the first of each: one function per name reaches the
/// file however many crossings produced it.
fn dedup_by_name(functions: Vec<syn::ItemFn>) -> Vec<(syn::Ident, syn::ItemFn)> {
    let mut by_name: BTreeMap<String, (syn::Ident, syn::ItemFn)> = BTreeMap::new();
    for function in functions {
        let name = function.sig.ident.clone();
        by_name.entry(name.to_string()).or_insert((name, function));
    }
    by_name.into_values().collect()
}
