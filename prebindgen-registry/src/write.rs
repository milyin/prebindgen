//! Rust file emission for the resolved `Registry`.
//!
//! `write_rust` takes the [`Assembly`] the adapter compiled — the frozen graph
//! of final artifacts the file is made of; renders each with the writer-owned
//! [`crate::RustWriter`], adds every per-item `on_<kind>` output and every
//! anonymous const, and hands the assembled file to `Destination::write`.
//!
//! The artifacts arrive from the adapter rather than being collected from the
//! registry, which is what lets one crossing contribute more than one function
//! — or one that occupies more than a single wire value.
//!
//! This module is `pub`, so **every `pub` item in it is public API of the
//! crate**. [`RustArtifact`] is the deliberately narrow late-rendering seam for
//! out-of-crate adapters. Every artifact carries registry-owned semantic
//! identity; final Rust names are never used as planning identity.

use std::{
    collections::{BTreeSet, HashMap},
    path::{Path, PathBuf},
};

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
/// Emission-integrity checks still run here because reachability is finalized
/// while per-item output is assembled.
#[derive(Debug)]
pub enum WriteError {
    /// Generated code calls a planned private converter whose function was
    /// removed by reachability filtering.
    UnrenderedConverterCalls {
        /// `(caller, missing converter)` pairs, sorted and de-duplicated.
        calls: Vec<(String, String)>,
    },
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::UnrenderedConverterCalls { calls } => {
                write!(
                    f,
                    "generated code calls private converters that were not rendered:"
                )?;
                for (caller, converter) in calls {
                    write!(f, "\n  - `{caller}` calls `{converter}`")?;
                }
                write!(
                    f,
                    "\nconverter reachability or dependency planning is incomplete"
                )
            }
        }
    }
}

impl std::error::Error for WriteError {}

/// Identity of one final artifact in an [`Assembly`].
///
/// A private converter is identified by the registry operation it implements;
/// every other final artifact — a helper the converter bodies call, an
/// exported wrapper, a type mirror — is identified by the adapter-scoped
/// [`ArtifactId`](crate::generation::ArtifactId) its plan was recorded under.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArtifactKey {
    /// A private converter, identified by its registry operation.
    Operation(crate::generation::OperationId),
    /// Any other final artifact, identified by its plan's artifact identity.
    Artifact(crate::generation::ArtifactId),
}

impl std::fmt::Display for ArtifactKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtifactKey::Operation(operation) => operation.fmt(f),
            ArtifactKey::Artifact(artifact) => artifact.fmt(f),
        }
    }
}

/// One final artifact of the generated Rust file.
///
/// Resolution decides what the artifact is and what it depends on; this trait
/// is the one thing the artifact still owes the writer — the Rust items it
/// becomes. [`Self::render`] runs at the final writing boundary, with the
/// [`crate::RustWriter`] used for the rest of final Rust emission and with no
/// access to a live [`Registry`], so an artifact cannot resume resolution
/// while the file is being assembled.
pub trait RustArtifact {
    /// Semantic identity, owned by the registry rather than by the Rust
    /// symbol final emission will choose.
    fn key(&self) -> ArtifactKey;

    /// Whether the generated adapter surface reaches this artifact.
    /// Validation-only artifacts return false: they are still rendered so
    /// that a caller of one can be reported, but they do not reach the file.
    fn reachable(&self) -> bool {
        true
    }

    /// Materialize the artifact's Rust items.
    fn render(&self, emit: &crate::RustWriter) -> Vec<syn::Item>;
}

/// The frozen set of final artifacts a generated Rust file is assembled from.
///
/// An assembly is ordered: artifacts reach the file in the order the adapter
/// added them, which for converters is the registry-owned dependency order of
/// the fragments they came from. It holds one artifact per [`ArtifactKey`], so
/// sharing never depends on the Rust symbol final emission allocates.
pub struct Assembly<A> {
    artifacts: Vec<A>,
}

impl<A: RustArtifact> Assembly<A> {
    /// The artifacts, in emission order. Includes the unreachable ones — the
    /// writer renders those to report anything that still calls them.
    pub fn artifacts(&self) -> impl ExactSizeIterator<Item = &A> {
        self.artifacts.iter()
    }
}

/// Collection phase preceding a frozen [`Assembly`].
pub struct AssemblyBuilder<A> {
    positions: HashMap<ArtifactKey, usize>,
    artifacts: Vec<A>,
}

impl<A> Default for AssemblyBuilder<A> {
    fn default() -> Self {
        Self {
            positions: HashMap::new(),
            artifacts: Vec::new(),
        }
    }
}

impl<A: RustArtifact> AssemblyBuilder<A> {
    /// Start an empty assembly.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add one final artifact.
    ///
    /// An identity already present is kept once. A reachable artifact replaces
    /// an unreachable one already held under the same identity, which is what
    /// lets an adapter add a dormant artifact before the parent that reaches
    /// it has been planned.
    pub fn artifact(&mut self, artifact: A) -> &mut Self {
        match self.positions.get(&artifact.key()).copied() {
            Some(position) => {
                if !self.artifacts[position].reachable() && artifact.reachable() {
                    self.artifacts[position] = artifact;
                }
            }
            None => {
                self.positions.insert(artifact.key(), self.artifacts.len());
                self.artifacts.push(artifact);
            }
        }
        self
    }

    /// Freeze the assembly.
    pub fn build(self) -> Assembly<A> {
        Assembly {
            artifacts: self.artifacts,
        }
    }
}

impl<A: RustArtifact> FromIterator<A> for Assembly<A> {
    fn from_iter<I: IntoIterator<Item = A>>(artifacts: I) -> Self {
        let mut builder = AssemblyBuilder::new();
        for artifact in artifacts {
            builder.artifact(artifact);
        }
        builder.build()
    }
}

/// Emit a resolved registry, assembling the generated Rust file from a frozen
/// [`Assembly`].
///
/// The assembly is what the adapter's compilation produced: one final artifact
/// per item the file will contain, already deduplicated and in registry-owned
/// dependency order. Handing artifacts over instead of a converter table is
/// what frees an adapter to emit a conversion the table could not hold —
/// several functions for one crossing, or one occupying more than a single
/// wire value.
///
/// `out_path` may be relative (resolved against `OUT_DIR` by prebindgen) or
/// absolute. Returns the path actually written.
pub fn write_rust<P: AsRef<Path>, E: Prebindgen, A: RustArtifact>(
    registry: &Registry,
    ext: &E,
    assembly: &Assembly<A>,
    out_path: P,
) -> Result<PathBuf, WriteError> {
    // Validation already ran ONCE in the generator's `build` — a built generator
    // (the only source of a resolved registry) is valid by construction, so
    // this writer does no binding resolution. It does validate the assembled
    // private call graph before handing the file to the destination.
    // The capability, minted here and nowhere else in this function's reach.
    // Every callback below is handed a borrow; nothing else in the pipeline is.
    // See `prebindgen_flat::flat::emit` for what that buys and what it
    // deliberately does not.
    let emit = crate::RustWriter::new(registry, ext.source_module());
    let mut items: Vec<syn::Item> = Vec::new();

    // 0. Adapter prerequisites — runtime-support items (helper structs,
    //    type aliases) the converter bodies depend on. Emitted first so
    //    everything below can reference them.
    items.extend(ext.prerequisites(registry, &emit));

    // 2. Per-item Rust output from the adapter — only for items the adapter
    //    explicitly declared. Undeclared items were already announced
    //    via `cargo:warning=` by the generator's own unclaimed-item report.
    //    Functions are not here: an exported wrapper is an artifact of the
    //    assembly above, planned when the adapter's generation plan was.
    let declared = registry.declared();
    let declared_types = &declared.types;
    let flat = registry.flat();
    let mut body_items: Vec<syn::Item> = Vec::new();
    body_items.extend(
        sorted_by_name(flat.types().filter_map(|t| match t {
            prebindgen_flat::flat::Type::Struct(s) => Some((&s.name, s)),
            _ => None,
        }))
        .into_iter()
        .filter(|(ident, _)| declared_types.contains_key(&TypeKey::from_ident(ident)))
        .flat_map(|(_, item)| ext.on_struct(item, registry, &emit)),
    );
    // Both enum shapes emit through `on_enum` and sort together: they were one
    // map here before they were two elements. They still SORT together — the
    // emission order is one sequence — but they dispatch to their own methods
    // now, because handing an adapter a `Type` it has to re-match is worse than
    // handing it the element the model already decided on.
    body_items.extend(
        sorted_by_name(flat.types().filter_map(|t| match t {
            prebindgen_flat::flat::Type::Variant(v) => Some((&v.name, t)),
            prebindgen_flat::flat::Type::Enum(e) => Some((&e.name, t)),
            _ => None,
        }))
        .into_iter()
        .filter(|(ident, _)| declared_types.contains_key(&TypeKey::from_ident(ident)))
        .flat_map(|(_, t)| match t {
            prebindgen_flat::flat::Type::Variant(v) => ext.on_variant(v, registry, &emit),
            prebindgen_flat::flat::Type::Enum(e) => ext.on_enum(e, registry, &emit),
            _ => unreachable!("filtered to the two enum shapes above"),
        }),
    );
    // Consts: an adapter WITH a const declaration mechanism
    // (`declared_consts() == Some(set)`) emits declared consts only,
    // symmetric with functions; an adapter without one (`None`) gets every
    // const through its own mandatory `on_const` policy. Prebindgen's
    // own injected feature guards are not consts at all — see the guards loop.
    let declared_consts = &declared.consts;
    body_items.extend(
        sorted_by_name(flat.constants().map(|c| (&c.name, c)))
            .into_iter()
            .filter(|(ident, _)| {
                declared_consts
                    .as_ref()
                    .is_none_or(|set| set.contains(*ident))
            })
            .flat_map(|(_, item)| ext.on_const(item, registry, &emit)),
    );

    // Every artifact renders, including the ones the adapter surface does not
    // reach: an unreachable artifact still contributes its function names, so
    // a caller that survived reachability filtering can be reported below.
    let rendered: Vec<(bool, Vec<syn::Item>)> = assembly
        .artifacts()
        .map(|artifact| (artifact.reachable(), artifact.render(&emit)))
        .collect();
    let converter_names: BTreeSet<String> = rendered
        .iter()
        .flat_map(|(_, artifact_items)| artifact_items)
        .filter_map(|item| match item {
            syn::Item::Fn(function) => Some(function.sig.ident.to_string()),
            _ => None,
        })
        .collect();
    for (reachable, artifact_items) in rendered {
        if reachable {
            items.extend(artifact_items);
        }
    }
    items.extend(body_items);

    // 3. Anonymous consts, verbatim. Last, and in stream order. Ungated on
    //    purpose: with no name there is nothing for an adapter to declare, so
    //    the const gate above cannot apply to them.
    for guard in flat.guards() {
        items.push(syn::Item::Const(emit.guard(guard)));
    }

    validate_converter_calls(&mut items, &converter_names)?;
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

/// Refuse a generated file whose rendered functions still call a private
/// converter plan removed by reachability filtering.
///
/// This is deliberately an integrity check for planned converter functions,
/// not a general unresolved-name check. Calls to missing prerequisites or
/// arbitrary external functions remain rustc's responsibility.
fn validate_converter_calls(
    items: &mut [syn::Item],
    candidates: &BTreeSet<String>,
) -> Result<(), WriteError> {
    let rendered: BTreeSet<String> = items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Fn(function) => Some(function.sig.ident.to_string()),
            _ => None,
        })
        .collect();
    let mut visitor = ConverterCallValidator {
        candidates,
        rendered: &rendered,
        caller: None,
        calls: BTreeSet::new(),
    };
    for item in items {
        syn::visit_mut::VisitMut::visit_item_mut(&mut visitor, item);
    }
    if visitor.calls.is_empty() {
        Ok(())
    } else {
        Err(WriteError::UnrenderedConverterCalls {
            calls: visitor.calls.into_iter().collect(),
        })
    }
}

struct ConverterCallValidator<'a> {
    candidates: &'a BTreeSet<String>,
    rendered: &'a BTreeSet<String>,
    caller: Option<String>,
    calls: BTreeSet<(String, String)>,
}

impl syn::visit_mut::VisitMut for ConverterCallValidator<'_> {
    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        let previous = self.caller.replace(function.sig.ident.to_string());
        syn::visit_mut::visit_item_fn_mut(self, function);
        self.caller = previous;
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            if let Some(segment) = path.path.segments.last() {
                let name = segment.ident.to_string();
                if self.candidates.contains(&name) && !self.rendered.contains(&name) {
                    self.calls.insert((
                        self.caller
                            .clone()
                            .unwrap_or_else(|| "<generated item>".to_string()),
                        name,
                    ));
                }
            }
        }
        syn::visit_mut::visit_expr_call_mut(self, call);
    }
}

#[cfg(test)]
mod tests;
