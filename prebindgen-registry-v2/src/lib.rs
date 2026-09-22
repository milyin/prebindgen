//! # prebindgen-registry-v2
//!
//! The **experimental v2 generation engine** of
//! [#719](https://github.com/milyin/prebindgen/issues/719), built beside the
//! shipping one rather than inside it.
//!
//! It accepts every input v1 accepts — the same captured sources, the same
//! declarations, the same settings — and generates the subset it has
//! implemented. Everything else is a **skip**: the declaration is not
//! generated, and [`Generation::skipped`] says which capability stopped it and
//! where. A missing implementation is an outcome of the run, never a fatal
//! build error, and never a reason to run v1 for that declaration.
//!
//! The language frontends drive it: under `PREBINDGEN_PIPELINE=v2` a
//! `prebindgen-c` or `prebindgen-jni` build hands [`generate`] its
//! declarations and its own [`Target`], and writes the [`Generation`] that
//! comes back. The user's `build.rs` is the same
//! under either engine.
//!
//! # What lives here, and what does not
//!
//! The registry owns the run: resolution, dependency closure, plans, artifact
//! identity and ordering, what survives. A language adapter states target
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
pub mod run;
pub mod target;
#[cfg(test)]
mod tests;

pub use body::{Instr, NodeBody, Operand, ValueId};
pub use decl::Declaration;
pub use outcome::{Capability, EngineError, Outcome, Skip};
pub use plan::{generate, FunctionPlan, NodeId, OutputId, ValuePlan};
pub use run::{Generation, PIPELINE};
pub use target::{
    mirrored_enum, AbiSpec, Access, Artifact, BoundarySpec, ChildValue, Crossing, Direction,
    EnumArm, FailureCategory, FailureRoute, Layout, OperandRole, OperandSpec, Operation,
    OperationType, OutputPlacement, ParamRole, Part, PlanningError, Position, PrimitiveFailure,
    PrimitiveId, PrimitiveSpec, Protocol, Relation, RelationId, ReprSpec, Requirement,
    ResolvedShape, ResolvedValues, Selection, SelectionQuery, SiteDescriptor, SourceItem,
    StandardOp, StructRelation, SurfaceRequest, SurfaceSpec, Target, TargetAttempt, TargetSupport,
    Terminal, Unsupported, WireType, WrapperParam,
};
