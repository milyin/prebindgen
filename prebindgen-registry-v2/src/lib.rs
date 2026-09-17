//! # prebindgen-registry-v2
//!
//! The **experimental v2 generation engine** of
//! [#719](https://github.com/milyin/prebindgen/issues/719), built beside the
//! shipping one rather than inside it.
//!
//! It accepts every input v1 accepts — the same captured sources, the same
//! declarations, the same settings — and generates the subset it has
//! implemented. Everything else is a **reported skip** with a stable capability
//! code and the path from the declaration to the site that could not be
//! lowered. A missing implementation is an outcome, never a fatal build error,
//! and never a reason to run v1 for that declaration.
//!
//! ```text
//! v2: 2 emitted, 45 skipped, 4 ignored (c target)
//! SKIP unsupported.c.opaque_ptr: fn:calculator_new, fn:calculator_apply (+18 more)
//! ```
//!
//! The language frontends drive it: under `PREBINDGEN_PIPELINE=v2` a
//! `prebindgen-c` or `prebindgen-jni` build turns its declarations into
//! [`BindingRequests`], hands them to [`generate`] with its own [`Target`], and
//! writes the [`Generation`] that comes back. The user's `build.rs` is the same
//! under either engine.
//!
//! # What lives here, and what does not
//!
//! The registry owns the run: resolution, dependency closure, plans, artifact
//! identity and ordering, the report. A language adapter states target
//! representations, naming and rendering — it does not own a type walk or a
//! scheduler of its own. See #719 §8.
//!
//! This crate does **not** depend on `prebindgen-registry`. It shares the
//! captured-source frontend (`prebindgen`, `prebindgen-flat`) and nothing else,
//! so "v2 never invokes the v1 planner" is a fact about the dependency graph
//! rather than a promise in a comment.
//!
//! # Selecting it
//!
//! Availability is the adapter's optional `v2` Cargo feature; selection is
//! `PREBINDGEN_PIPELINE=v2`, resolved once at `.build()` — see
//! [`prebindgen_flat::pipeline`].

pub mod body;
pub mod decl;
mod emit;
pub mod outcome;
pub mod plan;
pub mod report;
pub mod run;
pub mod target;
#[cfg(test)]
mod tests;

pub use body::{Instr, NodeBody, Operand, ValueId};
pub use decl::{Declaration, DeclarationId, DeclarationKind, SourceKind};
pub use outcome::{Capability, EngineError, Outcome, Skip};
pub use plan::{
    generate, BindingRequests, FunctionPlan, NodeId, OutputRequest, PolicyId, ValuePlan,
};
pub use report::{Counts, Report, SCHEMA_VERSION};
pub use run::{Generation, PIPELINE};
pub use target::{
    AbiSpec, Access, Artifact, BoundarySpec, ChildValue, Crossing, Direction, FailureCategory,
    FailureRoute, Layout, NativeParam, OperandRole, OperandSpec, Operation, OperationType,
    OutputPlacement, ParamRole, Part, PlanningError, Position, PrimitiveFailure, PrimitiveId,
    PrimitiveSpec, Protocol, RecordRelation, Relation, RelationId, ReprSpec, ResolvedShape,
    ResolvedValues, SelectionQuery, SiteDescriptor, SourceItem, StandardOp, SurfaceRequest,
    SurfaceSpec, Target, TargetAttempt, TargetSupport, Terminal, Unsupported, WireType,
};

/// A spelling with the spaces a reader does not want: `Option < Grade >` becomes
/// `Option<Grade>`, `# [cfg (unix)]` becomes `#[cfg(unix)]`, and the space in
/// `dyn Error` — the only kind that separates two words — stays.
///
/// Both callers arrive at the same problem from different directions: a
/// canonical type key spells a generic with spaces around its brackets, and a
/// `TokenStream` prints a space between every pair of tokens. Neither is wrong,
/// and neither is what a report or a doc comment should show.
pub fn close_up(spaced: &str) -> String {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let characters: Vec<char> = spaced.chars().collect();
    let mut out = String::with_capacity(spaced.len());
    for (index, &character) in characters.iter().enumerate() {
        if character == ' ' {
            let before = index.checked_sub(1).map(|i| characters[i]);
            let after = characters.get(index + 1).copied();
            let separates_words = before.is_some_and(word) && after.is_some_and(word);
            if !separates_words {
                continue;
            }
        }
        out.push(character);
    }
    out
}

#[cfg(test)]
mod close_up_tests {
    #[test]
    fn punctuation_closes_up_and_words_stay_apart() {
        for (key, expected) in [
            ("Option < Grade >", "Option<Grade>"),
            ("& [Payload]", "&[Payload]"),
            ("dyn Error", "dyn Error"),
            (
                "Result < Box < dyn Error > , u8 >",
                "Result<Box<dyn Error>,u8>",
            ),
            ("# [cfg (unix)]", "#[cfg(unix)]"),
            (
                "# [cfg (target_os = \"linux\")]",
                "#[cfg(target_os=\"linux\")]",
            ),
        ] {
            assert_eq!(super::close_up(key), expected, "{key}");
        }
    }
}
