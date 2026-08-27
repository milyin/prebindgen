//! Rust file emission for the resolved `Registry`.
//!
//! `write_rust` takes the [`Assembly`] the adapter compiled — the frozen set
//! of final artifacts the file is made of; renders each with the writer-owned
//! [`crate::RustWriter`], surrounds them with the adapter's prerequisites and
//! prebindgen's own anonymous feature-check consts, and hands the assembled
//! file to `Destination::write`.
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

use crate::{destination::Destination, prebindgen::Prebindgen, registry::Registry};

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
    /// A reachable artifact calls one that does not reach the file, either
    /// because reachability filtering dropped it or because it was never
    /// planned. Proven from the artifacts' own edges, before anything renders.
    UnreachedDependency {
        /// `(caller, unreached callee)` identities, sorted and de-duplicated.
        edges: Vec<(String, String)>,
    },
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::UnreachedDependency { edges } => {
                write!(f, "generated artifacts call artifacts the file omits:")?;
                for (caller, callee) in edges {
                    write!(f, "\n  - {caller} calls {callee}")?;
                }
                write!(
                    f,
                    "\nartifact reachability or dependency planning is incomplete"
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

    /// The identities this artifact's items answer for.
    ///
    /// Defaults to its own — the common case, where one artifact renders the
    /// items of one identity. An artifact that renders another identity's item
    /// too says so here, which is how a dependency on that identity is
    /// satisfied. The C callback artifact does exactly that: it renders the
    /// Invoke helper of a converter operation whose own fragment deliberately
    /// carries no artifact.
    fn provides(&self) -> Vec<ArtifactKey> {
        vec![self.key()]
    }

    /// The artifacts this one's body calls.
    ///
    /// Deliberately without a default: an artifact that calls nothing says so
    /// by returning an empty vector, and one that gains a call has to answer
    /// the question again. These edges are what proves the file complete —
    /// see [`WriteError::UnreachedDependency`] — so an unanswered one is a
    /// silently weaker check rather than a compile error.
    fn calls(&self) -> Vec<ArtifactKey>;

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
    /// Every identity the held artifacts answer for, including the ones an
    /// artifact provides beyond its own key.
    provided: HashMap<ArtifactKey, usize>,
}

impl<A: RustArtifact> Assembly<A> {
    /// The artifacts, in emission order, including the ones no reachable
    /// artifact calls. The writer skips those; they are held so that
    /// [`Self::reaches`] can tell "planned but unreached" from "never
    /// planned".
    pub fn artifacts(&self) -> impl ExactSizeIterator<Item = &A> {
        self.artifacts.iter()
    }

    /// Whether an artifact answering for this identity reaches the file.
    ///
    /// Reachability is read here rather than when the assembly was frozen: an
    /// adapter may reach an artifact after adding it, and the file is what
    /// settles the answer.
    pub fn reaches(&self, key: &ArtifactKey) -> bool {
        self.provided
            .get(key)
            .is_some_and(|position| self.artifacts[*position].reachable())
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
    ///
    /// Two *reachable* artifacts sharing one identity are a planning error,
    /// not a choice to make: keeping either would drop an item the file needs,
    /// and the reader would meet it as a missing name in the generated crate
    /// rather than as a diagnostic here. Converters are exempt — one
    /// registry operation is legitimately reached from several sites, and
    /// de-duplicating those is what this method is for.
    pub fn artifact(&mut self, artifact: A) -> &mut Self {
        match self.positions.get(&artifact.key()).copied() {
            Some(position) => {
                if let ArtifactKey::Artifact(id) = artifact.key() {
                    assert!(
                        !(self.artifacts[position].reachable() && artifact.reachable()),
                        "two reachable artifacts share the identity {id}"
                    );
                }
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
        let mut provided = self.positions;
        for (position, artifact) in self.artifacts.iter().enumerate() {
            for key in artifact.provides() {
                provided.insert(key, position);
            }
        }
        Assembly {
            artifacts: self.artifacts,
            provided,
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

/// Check an assembly's edges against what its artifacts render: every call to
/// another artifact's item is declared by [`RustArtifact::calls`], and every
/// identity [`RustArtifact::provides`] claims is one the artifact defines.
///
/// Test support, like [`RustWriter::for_test`](crate::RustWriter::for_test).
/// Emission proves the file complete from the edges alone, which is only worth
/// anything if the edges name every call — and the one place the calls
/// themselves are visible is the rendered output. So the adapters' own test
/// suites run this over the bindings they build: production reasons from the
/// edges, and this checks the edges against what rendering actually emits.
///
/// `namespace` is the adapter's operation namespace, the one it passes to
/// [`RustWriter::operation_ident`](crate::RustWriter::operation_ident).
///
/// # Panics
///
/// Naming the caller, the callee and the undeclared call, if a rendered body
/// calls a converter its artifact did not declare.
#[cfg(any(test, feature = "testing"))]
pub fn assert_edges_cover_rendered_calls<A: RustArtifact>(
    assembly: &Assembly<A>,
    emit: &crate::RustWriter,
    namespace: &str,
) {
    // What every artifact actually renders, which is the ground truth both
    // halves are checked against: a call resolves to an artifact by the name
    // that artifact defines, whether the identity is an operation or an
    // adapter-scoped artifact. A runtime helper is called by name like any
    // converter.
    let rendered: Vec<(ArtifactKey, BTreeSet<String>, BTreeSet<String>)> = assembly
        .artifacts()
        .map(|artifact| {
            let mut called = CalledIdents(BTreeSet::new());
            let mut defined = BTreeSet::new();
            for mut item in artifact.render(emit) {
                if let syn::Item::Fn(function) = &item {
                    defined.insert(function.sig.ident.to_string());
                }
                syn::visit_mut::VisitMut::visit_item_mut(&mut called, &mut item);
            }
            (artifact.key(), defined, called.0)
        })
        .collect();
    let mut definer: HashMap<&str, &ArtifactKey> = HashMap::new();
    for (key, defined, _) in &rendered {
        for name in defined {
            definer.insert(name.as_str(), key);
        }
    }
    let mut provider: HashMap<ArtifactKey, usize> = HashMap::new();
    for (position, artifact) in assembly.artifacts().enumerate() {
        for key in artifact.provides() {
            provider.insert(key, position);
        }
    }

    for (artifact, (key, defined, called)) in assembly.artifacts().zip(&rendered) {
        // Every identity claimed must be one this artifact actually defines,
        // or claiming it would grant reachability to an item no one renders.
        for claimed in artifact.provides() {
            if let ArtifactKey::Operation(operation) = &claimed {
                let ident = emit.operation_ident(namespace, operation).to_string();
                assert!(
                    defined.contains(&ident),
                    "{key} claims to provide {claimed}, but renders no `{ident}`"
                );
            }
        }
        if !artifact.reachable() {
            continue;
        }
        let mut allowed: BTreeSet<&str> = defined.iter().map(String::as_str).collect();
        for callee in artifact.calls() {
            // Resolved the way emission resolves it: through whichever
            // artifact answers for that identity, which is not always the one
            // whose own key it is.
            allowed.extend(
                provider
                    .get(&callee)
                    .into_iter()
                    .flat_map(|position| rendered[*position].1.iter().map(String::as_str)),
            );
        }
        for name in called {
            if allowed.contains(name.as_str()) {
                continue;
            }
            if let Some(callee) = definer.get(name.as_str()) {
                panic!(
                    "{key} calls {callee} as `{name}`, which it does not declare as a dependency"
                );
            }
        }
    }
}

/// Every function name called in the visited items.
#[cfg(any(test, feature = "testing"))]
struct CalledIdents(BTreeSet<String>);

#[cfg(any(test, feature = "testing"))]
impl syn::visit_mut::VisitMut for CalledIdents {
    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        if let syn::Expr::Path(path) = call.func.as_ref() {
            if let Some(segment) = path.path.segments.last() {
                self.0.insert(segment.ident.to_string());
            }
        }
        syn::visit_mut::visit_expr_call_mut(self, call);
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
    // Every artifact below is handed a borrow; nothing else in the pipeline is.
    // See `prebindgen_flat::flat::emit` for what that buys and what it
    // deliberately does not.
    let emit = crate::RustWriter::new(registry, ext.source_module());
    let mut items: Vec<syn::Item> = Vec::new();

    // Dependency completeness, proven from the artifacts' own edges before
    // anything is rendered: whatever a reachable artifact calls must itself
    // reach the file.
    let mut unreached: BTreeSet<(String, String)> = BTreeSet::new();
    for artifact in assembly.artifacts().filter(|artifact| artifact.reachable()) {
        for callee in artifact.calls() {
            if !assembly.reaches(&callee) {
                unreached.insert((artifact.key().to_string(), callee.to_string()));
            }
        }
    }
    if !unreached.is_empty() {
        return Err(WriteError::UnreachedDependency {
            edges: unreached.into_iter().collect(),
        });
    }

    // Only what reaches the file renders. An unreachable artifact used to be
    // rendered anyway, for its function names alone, so that a caller of one
    // could be named; the edges above answer that question without it.
    for artifact in assembly.artifacts().filter(|artifact| artifact.reachable()) {
        items.extend(artifact.render(&emit));
    }

    // 2. Anonymous consts, verbatim. Last, and in stream order. These are
    //    prebindgen's own injected feature checks, identical for every
    //    adapter and named by none, so they are written here rather than
    //    planned as an artifact of one adapter's assembly.
    for guard in registry.flat().guards() {
        items.push(syn::Item::Const(emit.guard(guard)));
    }

    let dest: Destination = items.into_iter().collect();
    Ok(dest.write(out_path))
}

#[cfg(test)]
mod tests;
