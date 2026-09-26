//! One finished v2 generation run: what was planned, what was emitted, and
//! what a missing capability left out.
//!
//! The run owns its whole pipeline — parsing, resolution, plans, emission and
//! the report. It reads the same model and the same declarations v1 does and
//! shares nothing else with it: no `Registry`, no fragment store, no resolver,
//! no emitter. That independence is the point of #719, and a dependency edge is
//! how it would be lost. [`crate::generate`] is the entry point; this is its
//! result.

use std::path::{Path, PathBuf};

use prebindgen_flat::{flat::Flat, Conditioned, RustEmitter};

use crate::{
    binding::{Binding, ReprId, Scope},
    decl::Declaration,
    outcome::{EngineError, Skip},
    plan::{FunctionPlan, Retained, ValuePlan},
    target::Target,
};

/// The engine's name wherever a run identifies itself.
pub const PIPELINE: &str = "v2";

/// A finished v2 run: the model and binding it read, what it generated, and
/// what a missing capability left out.
///
/// Immutable. Every writer is a pure emission over it, so they can run in any
/// order, or not at all.
pub struct Generation<T: Target> {
    flat: Flat,
    /// The target this was generated for, as [`Target::NAME`] spells it —
    /// stamped into the generated file.
    target: &'static str,
    binding: Binding<T>,
    skipped: Vec<(Declaration, Skip)>,
    unused_rules: Vec<(Scope, ReprId)>,
    values: Vec<ValuePlan>,
    retained: Vec<Retained>,
    functions: Vec<FunctionPlan<T::Op>>,
    rust: String,
}

impl<T: Target> std::fmt::Debug for Generation<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Generation({} target, {} function(s), {} skipped)",
            self.target,
            self.functions.len(),
            self.skipped.len()
        )
    }
}

impl<T: Target> Generation<T> {
    /// A finished run over the plans it retained.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        flat: Flat,
        target: &'static str,
        binding: Binding<T>,
        skipped: Vec<(Declaration, Skip)>,
        unused_rules: Vec<(Scope, ReprId)>,
        values: Vec<ValuePlan>,
        retained: Vec<Retained>,
        functions: Vec<FunctionPlan<T::Op>>,
        rust: String,
    ) -> Self {
        Generation {
            flat,
            target,
            binding,
            skipped,
            unused_rules,
            values,
            retained,
            functions,
            rust,
        }
    }

    /// The planned conversions, shared by every value that crosses the same
    /// way, by [`NodeId`](crate::NodeId).
    pub fn values(&self) -> &[ValuePlan] {
        &self.values
    }

    /// One planned conversion.
    pub fn value(&self, id: crate::NodeId) -> &ValuePlan {
        &self.values[id.0]
    }

    /// The outputs that survived, in the order the binding declared them, with
    /// the values planned for each — what a foreign writer declares.
    pub fn retained(&self) -> &[Retained] {
        &self.retained
    }

    /// The binding this run planned.
    pub fn binding(&self) -> &Binding<T> {
        &self.binding
    }

    /// The retained wrappers.
    pub fn functions(&self) -> &[FunctionPlan<T::Op>] {
        &self.functions
    }

    /// The generated Rust, as it is written to disk.
    pub fn rust(&self) -> &str {
        &self.rust
    }

    /// The model this run was planned over.
    pub fn flat(&self) -> &Flat {
        &self.flat
    }

    /// The `#[cfg]` conditions the captured item behind `declaration` was
    /// written under, spelled as the source wrote them — empty in the ordinary
    /// case.
    ///
    /// Text rather than tokens, because the registry is what puts a condition
    /// on the Rust it emits. What is left for a foreign writer is the
    /// declaration in *its* language, which usually cannot state a condition
    /// at all; saying so in that declaration's documentation is the most it
    /// can do.
    pub fn item_conditions(&self, declaration: &Declaration) -> Vec<String> {
        declaration
            .captured(&self.flat)
            .map(|element| crate::emit::Writer.conditions(Conditioned::Item(element)))
            .unwrap_or_default()
            .iter()
            .map(|condition| prebindgen_flat::close_up(&condition.to_string()))
            .collect()
    }

    /// What the binding asked for and did not get, in the order it asked.
    ///
    /// One entry per declaration a missing capability stopped, with the
    /// [`Skip`] naming the capability, a readable sentence, and the path from
    /// the declaration to the value that could not be lowered. A declaration
    /// absent from this list was generated.
    ///
    /// One entity may be declared more than once, so the same [`Declaration`]
    /// can appear here beside an emitted one: what the binding declared them
    /// as is what told them apart, and that is the target's own vocabulary,
    /// which this engine does not speak.
    pub fn skipped(&self) -> &[(Declaration, Skip)] {
        &self.skipped
    }

    /// The type rules no planned value was covered by, in the order the
    /// binding recorded them. Not an error: a type rule is a default for many
    /// values, and a scalar table covers kinds a binding may never mention.
    pub fn unused_rules(&self) -> &[(Scope, ReprId)] {
        &self.unused_rules
    }

    /// Write the generated Rust file.
    ///
    /// Stamped with the pipeline and target that produced it, so a file
    /// included by mistake from the other engine's output root says so on its
    /// first line rather than by a link error. The file is always valid Rust:
    /// a consumer's `include!` compiles whether this run generated anything or
    /// not.
    pub fn write_rust(&self, out_path: impl AsRef<Path>) -> Result<PathBuf, EngineError> {
        // A relative path is resolved against `OUT_DIR`, as v1 resolves one: a
        // build script that writes `"perftest.rs"` must not land a file in the
        // crate root under one engine and in `OUT_DIR` under the other.
        let out_path = out_path.as_ref();
        let out_path = if out_path.is_relative() {
            PathBuf::from(std::env::var("OUT_DIR").map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "a relative output path needs OUT_DIR, which only a build script sets",
                )
            })?)
            .join(out_path)
        } else {
            out_path.to_path_buf()
        };
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = format!(
            "// Generated by prebindgen {PIPELINE} ({} target) — do not edit.\n{}",
            self.target, self.rust
        );
        std::fs::write(&out_path, contents)?;
        Ok(out_path)
    }
}

/// Check the binding's outputs against the model before anything is planned.
///
/// A declaration is a statement of intent, so a declared item the source never
/// captured is an error and not a capability question — the same rule v1 holds.
///
/// Each declaration says which of the three captured kinds its name must find —
/// which is not always its own kind, since a Kotlin `val` may be backed by a
/// nullary function. Looking a function up among the constants reported a typo
/// that was not there; looking it up in the whole namespace let a
/// `.fun(fun!(x))` naming a captured `const x` through as a capability skip.
///
/// An output is the declaration *and* the form recorded with it, so one
/// entity declared twice is two outputs and is allowed; the same entity
/// declared twice in the same form is the binding saying one thing twice, and
/// is refused — the two would be one foreign declaration emitted from two
/// plans.
pub(crate) fn check_declarations<K: Clone + Eq + std::hash::Hash>(
    declared: &[(Declaration, K)],
    flat: &Flat,
) -> Result<(), EngineError> {
    let missing: Vec<_> = declared
        .iter()
        .map(|(declaration, _)| declaration)
        .filter(|declaration| declaration.missing_from(flat))
        .cloned()
        .collect();
    if !missing.is_empty() {
        return Err(EngineError::DeclaredNotFound { entries: missing });
    }

    let mut seen = std::collections::HashSet::new();
    let mut repeated: Vec<_> = declared
        .iter()
        .filter(|output| !seen.insert(*output))
        .map(|(declaration, _)| declaration.clone())
        .collect();
    repeated.sort();
    repeated.dedup();
    if !repeated.is_empty() {
        return Err(EngineError::DuplicateDeclaration { entries: repeated });
    }
    Ok(())
}

/// Whether a struct field was written under a `#[cfg]` the capture reader
/// could not answer.
///
/// A frontend asks while it builds a binding: a target that cannot state a
/// condition on its own side — Kotlin has none — refuses such a struct rather
/// than promise a member the library has only sometimes. Reading the
/// condition itself stays the registry's, which puts it on the Rust it emits.
pub fn field_is_conditional(field: &prebindgen_flat::flat::Field) -> bool {
    !crate::emit::Writer
        .conditions(Conditioned::Field(field))
        .is_empty()
}
