//! # prebindgen-registry-v2
//!
//! The **experimental v2 generation engine** of
//! [#719](https://github.com/milyin/prebindgen/issues/719), built beside the
//! shipping one rather than inside it.
//!
//! It accepts every input v1 accepts — the same captured sources, the same
//! declarations, the same settings — and generates the subset it has
//! implemented. Everything else is a **reported skip** with a stable capability
//! code and the path from the declared element to the site that could not be
//! lowered. A missing implementation is an outcome, never a fatal build error,
//! and never a reason to run v1 for that element.
//!
//! ```text
//! v2: 0 emitted, 45 skipped, 4 ignored (c target)
//! SKIP unsupported.function.not_implemented: fn:calculator_new, fn:calculator_apply (+18 more)
//! ```
//!
//! # What lives here, and what does not
//!
//! The registry owns the run: resolution, dependency closure, plans, artifact
//! identity and ordering, the manifest. A language adapter states target
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

pub mod decl;
pub mod outcome;
pub mod report;
pub mod run;

pub use decl::{BindingDeclarations, DeclaredElement, ElementId, ElementKind};
pub use outcome::{Capability, EngineError, Outcome, Skip};
pub use report::{Counts, Report, SCHEMA_VERSION};
pub use run::{plan, Generation, PIPELINE};
