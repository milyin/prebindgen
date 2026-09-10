//! What the engine asks a language adapter, and the shapes it answers in.
//!
//! Four planning questions — [`Target::select`], [`Target::represent`],
//! [`Target::boundary`], [`Target::surface`] — plus one rendering call,
//! [`Target::render_operation`]. Each planning answer is local: it describes
//! *this* value, *this* call, *this* declaration. The recursion that visits a
//! record's fields, the locals, the branches and the wrapper around them all
//! belong to the registry, which is why a nested record costs an adapter
//! nothing.
//!
//! Everything here is a **description**. Nothing an adapter returns is
//! generated text except [`Target::render_operation`]'s single expression and
//! the [`Artifact`]s it contributes whole.
//!
//! This is the first increment (docs/v2). What it carries is the scalar and
//! owned-record case; the fields the chapters describe for resources, validity
//! and sequences arrive with the capabilities that need them, and
//! `docs/v2/implementation.md` lists exactly what is absent.

use prebindgen_flat::flat::{Function, Struct, TypeRef};
use proc_macro2::TokenStream;

use crate::{
    decl::{DeclaredElement, ElementId},
    outcome::Capability,
};

/// Which way a value crosses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    /// Produce the Rust value a source function expects.
    IntoRust,
    /// Encode a Rust value for foreign code.
    OutOfRust,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::IntoRust => "into_rust",
            Direction::OutOfRust => "out_of_rust",
        }
    }
}

/// One value conversion to plan: an exact source type and a direction.
#[derive(Clone, Debug)]
pub struct Crossing {
    pub ty: TypeRef,
    pub direction: Direction,
}

/// A valid request needing a capability nobody has implemented yet.
///
/// Not an error: the run continues, the affected outputs are skipped, and the
/// report says which capability would unblock them.
#[derive(Clone, Debug)]
pub struct Unsupported {
    pub capability: Capability,
    pub explanation: String,
}

impl Unsupported {
    pub fn new(capability: impl Into<String>, explanation: impl Into<String>) -> Self {
        Unsupported {
            capability: Capability::new(capability),
            explanation: explanation.into(),
        }
    }
}

/// A target's answer: a description, or the capability it is missing.
#[derive(Debug)]
pub enum TargetAttempt<A> {
    Ready(A),
    Unsupported(Unsupported),
}

/// Every planning call answers with this: a description, a missing capability,
/// or a defect that fails the build.
pub type TargetSupport<A> = Result<TargetAttempt<A>, PlanningError>;

/// The input or the generator is wrong. Never a capability question.
#[derive(Clone, Debug)]
pub enum PlanningError {
    /// Malformed or contradictory configuration.
    InvalidInput(String),
    /// A generator defect or a violated internal contract.
    InternalInvariant(String),
}

impl std::fmt::Display for PlanningError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanningError::InvalidInput(message) => write!(f, "invalid input: {message}"),
            PlanningError::InternalInvariant(message) => {
                write!(f, "internal invariant: {message}")
            }
        }
    }
}

impl std::error::Error for PlanningError {}

// ---------------------------------------------------------------------------
// Relations: how a Rust value is built or read
// ---------------------------------------------------------------------------

/// A registered relation, valid inside one generation run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationId(pub(crate) usize);

/// One field of a record relation, or one argument of a constructor relation.
#[derive(Clone, Debug)]
pub struct Part {
    /// The field's name, absent for a positional field.
    pub name: Option<String>,
    /// Its position in the relation, which is its identity when it has no name.
    pub index: usize,
    /// Its exact source type.
    pub ty: TypeRef,
}

impl Part {
    /// How a diagnostic and a policy lookup address this part.
    pub fn label(&self) -> String {
        match &self.name {
            Some(name) => name.clone(),
            None => self.index.to_string(),
        }
    }
}

/// A record read through, or built from, its fields.
#[derive(Clone, Debug)]
pub struct RecordRelation {
    /// The record's declared name, which is how the writer finds its shape
    /// again when it renders a construction.
    pub record: String,
    pub parts: Vec<Part>,
}

/// How the registry constructs or reads a Rust value.
///
/// The first increment has the two the scalar and record paths need. A
/// constructor or projector relation is the same shape with different parts,
/// which is why adding one changes nothing below this type.
#[derive(Clone, Debug)]
pub enum Relation {
    /// The whole value converted by one target operation: no parts, no
    /// recursion.
    Atomic,
    /// The record's fields.
    Record(RecordRelation),
}

impl Relation {
    pub fn parts(&self) -> &[Part] {
        match self {
            Relation::Atomic => &[],
            Relation::Record(record) => &record.parts,
        }
    }

    /// The word a report uses for this relation.
    pub fn label(&self) -> String {
        match self {
            Relation::Atomic => "atomic".to_string(),
            Relation::Record(record) => format!("{}.fields", record.record),
        }
    }
}

// ---------------------------------------------------------------------------
// Target operations
// ---------------------------------------------------------------------------

/// A registered target/intermediate carrier type.
///
/// The Rust type is the adapter's own: a `repr(C)` aggregate it declares, a
/// `JObject`, a `jlong`. Source types are never spelled by an adapter — they
/// come from the model through the writer's emission capability.
#[derive(Clone, Debug)]
pub struct WireType {
    pub ty: syn::Type,
    /// Whether this carrier may appear in an extern signature. A Rust-only
    /// intermediate says no.
    pub abi: bool,
}

impl WireType {
    /// A carrier that may cross the ABI.
    pub fn abi(ty: syn::Type) -> Self {
        WireType { ty, abi: true }
    }

    /// A carrier used only inside generated Rust.
    pub fn internal(ty: syn::Type) -> Self {
        WireType { ty, abi: false }
    }
}

/// What an operand or result is.
#[derive(Clone, Debug)]
pub enum OperationType {
    /// An exact Rust type from the source model.
    Source(TypeRef),
    /// A target or intermediate carrier.
    Carrier(WireType),
}

/// What an operation is allowed to do with an operand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Owned,
    Shared,
    Exclusive,
}

/// Where an operand's value comes from.
///
/// A runtime context — the JNI environment — is an operand like any other, so
/// no rendered fragment can quietly depend on a variable its caller happens to
/// have named `env`. The boundary names the wrapper parameter that supplies
/// each context; the registry binds them at assembly.
#[derive(Clone, Debug)]
pub enum OperandRole {
    /// The value being converted, supplied by the enclosing conversion.
    Value,
    /// A runtime context the boundary provides under this name.
    Context(String),
    /// The error value, on a failure route only.
    Error,
}

#[derive(Clone, Debug)]
pub struct OperandSpec {
    pub ty: OperationType,
    pub access: Access,
    pub role: OperandRole,
}

impl OperandSpec {
    pub fn value(ty: OperationType, access: Access) -> Self {
        OperandSpec {
            ty,
            access,
            role: OperandRole::Value,
        }
    }

    pub fn context(name: impl Into<String>, ty: OperationType, access: Access) -> Self {
        OperandSpec {
            ty,
            access,
            role: OperandRole::Context(name.into()),
        }
    }

    pub fn error(ty: OperationType) -> Self {
        OperandSpec {
            ty,
            access: Access::Owned,
            role: OperandRole::Error,
        }
    }
}

/// Which boundary route a failure takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FailureCategory {
    /// Reported by an explicitly selected source operation.
    Domain,
    /// An invalid foreign value or a failed representation conversion.
    Binding,
    /// A target runtime failure, such as a JNI call.
    Runtime,
}

impl FailureCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            FailureCategory::Domain => "domain",
            FailureCategory::Binding => "binding",
            FailureCategory::Runtime => "runtime",
        }
    }
}

/// Whether performing an operation can fail, and with what.
#[derive(Clone, Debug)]
pub enum PrimitiveFailure {
    Infallible,
    Fallible {
        /// Boxed because a carrier holds a `syn::Type`, and an infallible
        /// operation would otherwise pay for one.
        error: Box<OperationType>,
        category: FailureCategory,
    },
}

impl PrimitiveFailure {
    /// An operation that can fail, with the typed error it supplies.
    pub fn fallible(error: OperationType, category: FailureCategory) -> Self {
        PrimitiveFailure::Fallible {
            error: Box::new(error),
            category,
        }
    }
}

/// An operation the registry itself knows how to render, so a target that
/// needs one ships no renderer for it.
#[derive(Clone, Debug)]
pub enum StandardOp {
    /// The result *is* the operand: the carrier and the source value are one
    /// Rust value. Renders nothing at all.
    Identity,
    /// Read a member of an aggregate carrier: `arg0.secs`.
    ReadMember { member: syn::Member },
}

/// What actually performs an operation.
///
/// This is the settled form of the chapters' open question about
/// registry-supplied operations inside an adapter's payload: a payload is the
/// adapter's own type, and the standard operations are a variant beside it
/// rather than something an adapter has to encode.
#[derive(Clone, Debug)]
pub enum Operation<P> {
    Standard(StandardOp),
    Target(P),
}

/// One typed operation supplied by a target.
///
/// It describes an operation, not a use of one: it names no variable and
/// belongs to no exported function, so the same description is applied wherever
/// the operation is needed.
#[derive(Clone, Debug)]
pub struct PrimitiveSpec<P> {
    pub operands: Vec<OperandSpec>,
    /// The value available after success, or nothing for an operation that
    /// produces none.
    pub result: Option<OperationType>,
    pub failure: PrimitiveFailure,
    /// Generated units this operation needs in order to compile.
    pub dependencies: Vec<Artifact>,
    pub implementation: Operation<P>,
}

impl<P> PrimitiveSpec<P> {
    /// The conversion that renders nothing: the carrier already *is* the source
    /// value.
    pub fn identity(ty: OperationType) -> Self {
        PrimitiveSpec {
            operands: vec![OperandSpec::value(ty.clone(), Access::Owned)],
            result: Some(ty),
            failure: PrimitiveFailure::Infallible,
            dependencies: Vec::new(),
            implementation: Operation::Standard(StandardOp::Identity),
        }
    }

    /// How many operands this application takes from the enclosing conversion.
    pub(crate) fn value_operands(&self) -> usize {
        self.operands
            .iter()
            .filter(|operand| matches!(operand.role, OperandRole::Value))
            .count()
    }

    /// Whether this application consumes the value it is given.
    pub(crate) fn consumes_value(&self) -> bool {
        self.operands.iter().any(|operand| {
            matches!(operand.role, OperandRole::Value) && operand.access == Access::Owned
        })
    }

    pub(crate) fn failure_category(&self) -> Option<FailureCategory> {
        match &self.failure {
            PrimitiveFailure::Infallible => None,
            PrimitiveFailure::Fallible { category, .. } => Some(*category),
        }
    }
}

/// A registered operation, valid inside one generation run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PrimitiveId(pub(crate) usize);

/// One generated unit: a helper function, a type declaration.
///
/// Contributed whole by whoever needs it. The name is its identity, so two
/// operations depending on the same helper emit it once.
#[derive(Clone, Debug)]
pub struct Artifact {
    pub name: String,
    pub rust: TokenStream,
}

impl Artifact {
    pub fn new(name: impl Into<String>, rust: TokenStream) -> Self {
        Artifact {
            name: name.into(),
            rust,
        }
    }
}

// ---------------------------------------------------------------------------
// Representations
// ---------------------------------------------------------------------------

/// The values that carry a representation.
#[derive(Clone, Debug)]
pub enum Layout {
    /// One carrier value.
    Scalar(WireType),
    /// One containing carrier with named members.
    Aggregate {
        ty: WireType,
        members: Vec<syn::Member>,
    },
}

impl Layout {
    /// The carrier a native signature would name for this layout.
    pub fn wire(&self) -> &WireType {
        match self {
            Layout::Scalar(wire) => wire,
            Layout::Aggregate { ty, .. } => ty,
        }
    }
}

/// The operations through which the registry reads or builds a representation.
#[derive(Clone, Debug)]
pub enum Protocol<P> {
    /// The whole value in one operation, with no parts.
    Terminal { codec: Box<PrimitiveSpec<P>> },
    /// One projection per part of the selected relation, in part order.
    Product { projections: Vec<PrimitiveSpec<P>> },
}

impl<P> Protocol<P> {
    /// The whole value converted by one operation.
    pub fn terminal(codec: PrimitiveSpec<P>) -> Self {
        Protocol::Terminal {
            codec: Box::new(codec),
        }
    }
}

/// A target's answer to "what carries this value, and how is it accessed".
#[derive(Clone, Debug)]
pub struct ReprSpec<P> {
    pub layout: Layout,
    pub protocol: Protocol<P>,
}

// ---------------------------------------------------------------------------
// The boundary
// ---------------------------------------------------------------------------

/// What a native parameter is for.
#[derive(Clone, Debug)]
pub enum ParamRole {
    /// Feeds the conversion of this source parameter.
    Input(usize),
    /// Supplies a runtime context operations ask for by this name.
    Context(String),
    /// Required by the calling convention and used by nothing.
    Unused,
}

#[derive(Clone, Debug)]
pub struct NativeParam {
    pub name: syn::Ident,
    pub ty: WireType,
    pub role: ParamRole,
    /// Whether the parameter binding needs `mut`.
    pub mutable: bool,
}

/// The native interface of one exported function.
#[derive(Clone, Debug)]
pub struct AbiSpec {
    /// The `extern` string: `"C"`, `"system"`.
    pub abi: String,
    /// The exported symbol.
    pub symbol: String,
    pub params: Vec<NativeParam>,
    /// The native return type, absent for a function returning nothing.
    pub ret: Option<WireType>,
}

/// Where a converted result goes.
#[derive(Clone, Debug)]
pub enum OutputPlacement {
    /// This path delivers no value.
    Void,
    /// Through the native return.
    Return,
}

/// How a failure route ends the native call.
#[derive(Clone, Debug)]
pub enum Terminal {
    /// Return this expression to the caller.
    Return(syn::Expr),
    /// Give up: the process cannot continue correctly.
    Abort,
}

/// The terminal action for one category of failure.
#[derive(Clone, Debug)]
pub struct FailureRoute<P> {
    pub category: FailureCategory,
    /// An operation that reports the error before terminating, if the target
    /// has one. Its `Error` operand receives the failed operation's error.
    pub report: Option<PrimitiveSpec<P>>,
    /// What to do when reporting itself fails.
    pub on_report_failure: Terminal,
    /// How this route ends.
    pub terminate: Terminal,
}

/// A target's answer for one exported function's native interface.
#[derive(Clone, Debug)]
pub struct BoundarySpec<P> {
    pub abi: AbiSpec,
    pub output: OutputPlacement,
    /// One route per failure category the conversions can raise. A category
    /// with no route makes the function unsupported; its ABI is never quietly
    /// changed to fit.
    pub failures: Vec<FailureRoute<P>>,
}

// ---------------------------------------------------------------------------
// Public declarations
// ---------------------------------------------------------------------------

/// A target's description of one public declaration and what it requires.
#[derive(Clone, Debug)]
pub struct SurfaceSpec<P> {
    pub element: ElementId,
    /// Other requested elements that must be emitted for this one to be.
    pub requires: Vec<ElementId>,
    /// Rust this declaration contributes: the `repr(C)` aggregate a C caller
    /// fills in, for instance. A target whose public declaration is not Rust
    /// contributes none.
    pub rust: Vec<Artifact>,
    /// What the target's own writer needs to render its foreign declaration.
    pub payload: Option<P>,
}

// ---------------------------------------------------------------------------
// What the registry hands a target
// ---------------------------------------------------------------------------

/// Where a conversion sits, for policy lookup and for diagnostics.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    /// The requested output this conversion is reached from.
    pub element: ElementId,
    /// Root to here: `param 0`, then `field secs`.
    pub path: Vec<String>,
}

impl Position {
    pub fn root(element: ElementId) -> Self {
        Position {
            element,
            path: Vec::new(),
        }
    }

    pub fn child(&self, step: impl Into<String>) -> Self {
        let mut path = self.path.clone();
        path.push(step.into());
        Position {
            element: self.element.clone(),
            path,
        }
    }

    /// This position as a report's dependency path.
    pub fn dependency_path(&self) -> Vec<String> {
        std::iter::once(self.element.to_string())
            .chain(self.path.iter().cloned())
            .collect()
    }
}

/// What `select` is given: the value, the relations available for it, and the
/// choices recorded for this position.
pub struct SelectionQuery<'a, Policy> {
    pub crossing: &'a Crossing,
    pub position: &'a Position,
    /// The relations registered for this type: the implicit one, plus any the
    /// configuration declared. A target picks one; it does not invent one.
    ///
    /// There is one per type today, because every relation the increment has is
    /// implicit. Pinning a relation against the target's choice arrives with the
    /// second kind — a declared constructor — since with one candidate there is
    /// nothing to pin.
    pub candidates: &'a [(RelationId, Relation)],
    pub policy: &'a Policy,
}

/// What `represent` is given: the selected relation, resolved against the
/// model.
pub struct ResolvedShape<'a> {
    pub crossing: &'a Crossing,
    pub position: &'a Position,
    pub relation: &'a Relation,
    /// The record behind a record relation, for a target that renders its own
    /// declaration of it.
    pub record: Option<&'a Struct>,
}

/// One already-planned child, as its parent's representation sees it.
pub struct ChildValue<'a> {
    pub part: &'a Part,
    pub layout: &'a Layout,
}

/// The conversions of one exported function, as the boundary and the public
/// declaration see them.
pub struct ResolvedValues<'a, P> {
    pub inputs: Vec<&'a crate::plan::ValuePlan<P>>,
    pub output: Option<&'a crate::plan::ValuePlan<P>>,
}

impl<P> ResolvedValues<'_, P> {
    /// Every failure category the planned conversions can raise, sorted.
    pub fn failure_categories(&self) -> Vec<FailureCategory> {
        let mut categories: Vec<_> = self
            .inputs
            .iter()
            .chain(self.output.iter())
            .flat_map(|value| value.failures.iter().copied())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }
}

/// What `boundary` is given: the exported call and its source signature.
pub struct SiteDescriptor<'a> {
    pub element: &'a DeclaredElement,
    pub function: &'a Function,
}

/// What `surface` is given: the requested public declaration.
pub struct SurfaceRequest<'a, Policy> {
    pub element: &'a DeclaredElement,
    pub policy: &'a Policy,
    pub item: SourceItem<'a>,
}

/// The source item behind a requested output.
#[derive(Clone, Copy)]
pub enum SourceItem<'a> {
    Function(&'a Function),
    Record(&'a Struct),
}

// ---------------------------------------------------------------------------
// The interface
// ---------------------------------------------------------------------------

/// The language adapter, as the engine sees it.
///
/// Every method answers a local question about one value, one call or one
/// declaration. None of them walks a type, allocates a name, or decides control
/// flow: those are the registry's, which is why a second field, a nested record
/// or a third target costs an adapter nothing new.
pub trait Target {
    /// The choices the frontend recorded, in whatever shape this language's
    /// configuration takes. The engine carries it and hands it back.
    type Policy;
    /// The adapter's own rendering data, retained in the plans and read back by
    /// [`Target::render_operation`] and by the adapter's foreign writer.
    ///
    /// `Clone` because one described operation is applied wherever it is
    /// needed: the description is registered once per use site, and the
    /// alternative is an adapter allocating its own identities for the registry
    /// to trust.
    type Payload: Clone;

    /// Choose the relation for this value from the ones available.
    fn select(&self, query: &SelectionQuery<'_, Self::Policy>) -> TargetSupport<RelationId>;

    /// Describe what carries this value and how its parts are accessed.
    fn represent(
        &self,
        shape: &ResolvedShape<'_>,
        children: &[ChildValue<'_>],
        policy: &Self::Policy,
    ) -> TargetSupport<ReprSpec<Self::Payload>>;

    /// Describe this exported function's native interface and failure routes.
    fn boundary(
        &self,
        site: &SiteDescriptor<'_>,
        values: &ResolvedValues<'_, Self::Payload>,
        policy: &Self::Policy,
    ) -> TargetSupport<BoundarySpec<Self::Payload>>;

    /// Describe one public declaration and what it requires.
    fn surface(
        &self,
        request: &SurfaceRequest<'_, Self::Policy>,
        values: &ResolvedValues<'_, Self::Payload>,
    ) -> TargetSupport<SurfaceSpec<Self::Payload>>;

    /// Render one of this target's operations as a single Rust expression.
    ///
    /// One expression and nothing around it: no `let`, no `match`, no return.
    /// The operands arrive already named by the writer, in the order the
    /// operation's specification lists them.
    fn render_operation(&self, payload: &Self::Payload, operands: &[syn::Ident]) -> TokenStream;
}
