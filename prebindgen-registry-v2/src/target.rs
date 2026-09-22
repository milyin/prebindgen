//! What the engine asks a language adapter, and the shapes it answers in.
//!
//! Four planning questions — [`Target::select`], [`Target::represent`],
//! [`Target::boundary`], [`Target::surface`] — plus one rendering call,
//! [`Target::render_operation`]. Each planning answer is local: it describes
//! *this* value, *this* call, *this* declaration. The recursion that visits a
//! struct's fields, the locals, the branches and the wrapper around them all
//! belong to the registry, which is why a nested struct costs an adapter
//! nothing.
//!
//! Everything here is a **description**. Nothing an adapter returns is
//! generated text except [`Target::render_operation`]'s single expression and
//! the [`Artifact`]s it contributes whole.
//!
//! # The words the doc comments use
//!
//! - The **binding** is the crate a user builds and ships — `zenoh-flat-c`,
//!   `zenoh-flat-jni` — whose `build.rs` declares what to expose. It is the
//!   consumer of everything here.
//! - The **target** is a language together with the calling interface it
//!   reaches Rust through — C, or Kotlin through JNI — and, in code, the
//!   adapter implementing [`Target`] for it. The same crate is the
//!   **frontend** when it faces the binding's `build.rs`, and the target
//!   adapter when it faces the registry.
//! - The **registry** is this crate: it plans and asks; the target answers.
//! - A **source function** is a Rust function in the source crate — the
//!   `#[prebindgen]`-marked crate the captures come from — that the binding
//!   asked to expose, with `fun!(ledger_open)` or the like. It is never
//!   exported itself; it stays where it is and is called. An **exported
//!   function**, or **wrapper**, is the Rust function the registry generates
//!   around it: one the target's calling interface can reach, which converts
//!   what arrives, calls the source function once, and converts what it
//!   returns. The writer renders every wrapper as an `extern` function under a
//!   `#[no_mangle]` symbol; what a target decides of its form is in
//!   [`AbiSpec`] — the convention, the symbol, the signature, its attributes
//!   and whether it is `unsafe`. A binding exports only wrappers.
//! - The **wrapper boundary** is the form the wrapper is called through: its
//!   calling convention, symbol, parameters and return. A **wrapper
//!   parameter** is one of those parameters, spelled as a `jlong` or a
//!   `*mut ledger_t`. Everything on the other side of that boundary — the
//!   Kotlin function that calls the wrapper, the C prototype a caller
//!   compiles against — is the **foreign** or **public** side.
//!
//! This is the first increment (docs/v2). What it carries is the scalar, the
//! owned struct and the owned opaque handle; the fields the chapters describe
//! for resources, validity and sequences arrive with the capabilities that need
//! them, and `docs/v2/implementation.md` lists exactly what is absent.

use prebindgen_flat::{
    flat::{Extern, Function, Struct, TypeKind, TypeRef},
    Conditioned, RustEmitter,
};
use proc_macro2::TokenStream;

use crate::{decl::Declaration, outcome::Capability};

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

/// One field of a struct relation, or one argument of a constructor relation.
#[derive(Clone, Debug)]
pub struct Part {
    /// The field's name, absent for a positional field.
    pub name: Option<String>,
    /// Its position in the relation, which is its identity when it has no name.
    pub index: usize,
    /// Its exact source type.
    pub ty: TypeRef,
    /// The `#[cfg]` conditions the source field was written under, which the
    /// capture reader could not answer — empty in the ordinary case.
    ///
    /// A target re-declaring this part as a member of its own struct puts them
    /// on that member. It must not read them: the registry puts the same
    /// conditions on every instruction that serves this part, so a member
    /// declared under them is read under them.
    pub conditions: Vec<TokenStream>,
}

impl Part {
    /// How a diagnostic and a target's configuration address this part.
    pub fn label(&self) -> String {
        match &self.name {
            Some(name) => name.clone(),
            None => self.index.to_string(),
        }
    }
}

/// A struct read through, or built from, its fields.
#[derive(Clone, Debug)]
pub struct StructRelation {
    /// The struct's declared name, which is how the writer finds its shape
    /// again when it renders a construction.
    pub name: String,
    pub parts: Vec<Part>,
}

/// How the registry constructs or reads a Rust value.
///
/// The first increment has the two the scalar and struct paths need. A
/// constructor or projector relation is the same shape with different parts,
/// which is why adding one changes nothing below this type.
#[derive(Clone, Debug)]
pub enum Relation {
    /// The whole value converted by one target operation: no parts, no
    /// recursion.
    Atomic,
    /// The struct's fields.
    Struct(StructRelation),
}

impl Relation {
    pub fn parts(&self) -> &[Part] {
        match self {
            Relation::Atomic => &[],
            Relation::Struct(strukt) => &strukt.parts,
        }
    }

    /// The word a report uses for this relation.
    pub fn label(&self) -> String {
        match self {
            Relation::Atomic => "atomic".to_string(),
            Relation::Struct(strukt) => format!("{}.fields", strukt.name),
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
///
/// The three handle operations are here because they are Rust, not C or JNI:
/// moving a source value onto the heap and back is the same for every target,
/// and it is the one place a conversion has to spell a *source* type — which
/// only the registry may do. What differs per target is the carrier the
/// address is cast to, and that the target states.
#[derive(Clone, Debug)]
pub enum StandardOp {
    /// The result *is* the operand: the carrier and the source value are one
    /// Rust value. Renders nothing at all.
    Identity,
    /// Read a member of an aggregate carrier: `arg0.secs`.
    ReadMember { member: syn::Member },
    /// Hand an owned source value to the foreign side as an address:
    /// `Box::into_raw(Box::new(v)) as <carrier>`. The foreign side owns the
    /// allocation from here on.
    IntoRaw { carrier: Box<syn::Type> },
    /// Take back what [`StandardOp::IntoRaw`] handed out:
    /// `*Box::from_raw(v as *mut <source>)`. A null address is a binding
    /// failure carrying a `String`; the handle is consumed either way.
    FromRaw { source: Box<TypeRef> },
    /// Drop what [`StandardOp::IntoRaw`] handed out without converting it. A
    /// null address releases nothing, as `free(NULL)` does. Produces no value.
    Release { source: Box<TypeRef> },
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
/// belongs to no wrapper, so the same description is applied wherever the
/// operation is needed.
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

    /// Hand an owned `source` value out as `carrier` — see
    /// [`StandardOp::IntoRaw`]. Infallible.
    pub fn into_raw(source: TypeRef, carrier: WireType) -> Self {
        PrimitiveSpec {
            operands: vec![OperandSpec::value(
                OperationType::Source(source),
                Access::Owned,
            )],
            result: Some(OperationType::Carrier(carrier.clone())),
            failure: PrimitiveFailure::Infallible,
            dependencies: Vec::new(),
            implementation: Operation::Standard(StandardOp::IntoRaw {
                carrier: Box::new(carrier.ty),
            }),
        }
    }

    /// Take an owned `source` value back from `carrier` — see
    /// [`StandardOp::FromRaw`]. Fails with a `String` in the binding category
    /// when the address is null, so a boundary passing handles needs a route
    /// for that category.
    pub fn from_raw(carrier: WireType, source: TypeRef) -> Self {
        PrimitiveSpec {
            operands: vec![OperandSpec::value(
                OperationType::Carrier(carrier),
                Access::Owned,
            )],
            result: Some(OperationType::Source(source.clone())),
            failure: PrimitiveFailure::fallible(
                OperationType::Carrier(WireType::internal(syn::parse_quote!(String))),
                FailureCategory::Binding,
            ),
            dependencies: Vec::new(),
            implementation: Operation::Standard(StandardOp::FromRaw {
                source: Box::new(source),
            }),
        }
    }

    /// Drop a `source` value held behind `carrier` — see
    /// [`StandardOp::Release`]. Infallible, produces nothing.
    pub fn release(carrier: WireType, source: TypeRef) -> Self {
        PrimitiveSpec {
            operands: vec![OperandSpec::value(
                OperationType::Carrier(carrier),
                Access::Owned,
            )],
            result: None,
            failure: PrimitiveFailure::Infallible,
            dependencies: Vec::new(),
            implementation: Operation::Standard(StandardOp::Release {
                source: Box::new(source),
            }),
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
    /// The carrier a wrapper signature would name for this layout.
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
    /// How the foreign side gives a value in this representation back without
    /// converting it — the typed drop of a handle. Stated on the into-Rust
    /// representation, whose carrier it takes; `None` for a representation the
    /// foreign side holds by value and owes nothing for.
    ///
    /// The registry plans one exported release per declared type that has one,
    /// through [`Target::boundary`] with no source function, so a caller that
    /// never passes the handle back still has a way to free it.
    pub release: Option<PrimitiveSpec<P>>,
}

// ---------------------------------------------------------------------------
// The boundary
// ---------------------------------------------------------------------------

/// What a wrapper parameter is for.
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
pub struct WrapperParam {
    pub name: syn::Ident,
    pub ty: WireType,
    pub role: ParamRole,
    /// Whether the parameter binding needs `mut`.
    pub mutable: bool,
}

/// The boundary of one exported function: how the wrapper the
/// registry generates around a source function is reached from the target's
/// side.
///
/// The writer renders every wrapper as
/// `#[no_mangle] <attrs> pub <unsafe> extern "<abi>" fn <symbol>(<params>) -> <ret>`,
/// and everything in angle brackets is the target's to decide here. The
/// `#[no_mangle]`, the `pub` and the `fn` are the writer's: a wrapper is
/// reached by its symbol, and a target that needed another linkage would need
/// a variant of this type rather than an attribute that fights the one the
/// writer adds — which the registry refuses (see [`AbiSpec::attrs`]).
///
/// This is the specification's `AbiSpec<Payload>` with the payload made
/// concrete: the "target signature requirements" the chapter leaves open are
/// the attributes and the safety of the signature, because those are what the
/// writer has to render and can check, and what V1's JNI wrapper —
/// `#[allow(..)] pub unsafe extern "C"` — needs beyond the convention and
/// the symbol.
#[derive(Clone, Debug)]
pub struct AbiSpec {
    /// The `extern` string: `"C"`, `"system"`.
    pub abi: String,
    /// The symbol the wrapper is exported under, which the foreign side links
    /// against. Not the source function's name: the frontend's naming settled
    /// it.
    pub symbol: String,
    /// The wrapper's parameters, in boundary order — the ones serving a source
    /// parameter and the ones the calling convention adds.
    pub params: Vec<WrapperParam>,
    /// The wrapper's return type, absent when it returns nothing.
    pub ret: Option<WireType>,
    /// Attributes the wrapper carries beyond `#[no_mangle]`, rendered after
    /// it: a lint the generated signature would otherwise trip
    /// (`#[allow(non_snake_case)]`), a target's own marker. Empty for both
    /// targets today.
    ///
    /// Linkage is not the target's to restate: `#[no_mangle]` and
    /// `#[export_name]` here are contradictory input and fail the build, since
    /// the writer already exports the wrapper under [`AbiSpec::symbol`].
    pub attrs: Vec<syn::Attribute>,
    /// Whether the wrapper is an `unsafe fn`. `false` for both targets today:
    /// the wrapper checks what it is handed, and a caller owes it nothing a
    /// safe function cannot state. A convention that makes the caller
    /// responsible for the pointers it passes says so here, and the foreign
    /// declaration should say the same.
    pub unsafety: bool,
}

/// Where a converted result goes.
#[derive(Clone, Debug)]
pub enum OutputPlacement {
    /// This path delivers no value.
    Void,
    /// Through the wrapper's return.
    Return,
}

/// How a failure route ends the wrapper's call.
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

/// A target's answer to [`Target::boundary`]: the boundary of the
/// wrapper that will export one source function, and what the wrapper does
/// when a conversion inside it fails.
#[derive(Clone, Debug)]
pub struct BoundarySpec<P> {
    pub abi: AbiSpec,
    /// Where the converted result of the source call goes.
    pub output: OutputPlacement,
    /// One route per failure category the conversions can raise. A category
    /// with no route makes the export unsupported — the declaration is skipped
    /// — rather than the ABI being quietly changed to fit.
    pub failures: Vec<FailureRoute<P>>,
}

// ---------------------------------------------------------------------------
// Public declarations
// ---------------------------------------------------------------------------

/// A public type another declaration depends on.
///
/// A wrapper taking an aggregate is unusable unless the type it names is
/// emitted too. A target states that with the type, not with the id of the
/// declaration representing it: at [`Target::surface`] it holds the model's
/// answer about a value, not the binding's declaration of a type, and the
/// engine is the side that knows which declaration covers which type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Requirement {
    type_name: String,
}

impl Requirement {
    /// What `ty` requires, if it names a type at all — `None` for a scalar or
    /// anything else with no declared type of its own.
    pub fn of(ty: &TypeRef) -> Option<Self> {
        match ty.kind() {
            TypeKind::Named { id, .. } => Some(Requirement {
                type_name: id.name.clone(),
            }),
            _ => None,
        }
    }

    /// The same for a type the target knows by name without holding a
    /// reference to it.
    pub fn type_named(name: impl Into<String>) -> Self {
        Requirement {
            type_name: name.into(),
        }
    }

    /// The type's name, as the model spells it.
    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

impl std::fmt::Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "type `{}`", self.type_name)
    }
}

/// A target's description of one public declaration and what it requires.
#[derive(Clone, Debug)]
pub struct SurfaceSpec<P> {
    pub declaration: Declaration,
    /// Types that must be declared and emitted for this declaration to be.
    pub requires: Vec<Requirement>,
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

/// Where a conversion sits: what a target looks a per-site choice up by, and
/// what a diagnostic prints.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Position {
    /// The requested output this conversion is reached from.
    pub declaration: Declaration,
    /// Root to here: `param 0`, then `field secs`.
    pub path: Vec<String>,
}

impl Position {
    pub fn root(declaration: Declaration) -> Self {
        Position {
            declaration,
            path: Vec::new(),
        }
    }

    pub fn child(&self, step: impl Into<String>) -> Self {
        let mut path = self.path.clone();
        path.push(step.into());
        Position {
            declaration: self.declaration.clone(),
            path,
        }
    }

    /// This position as a report's dependency path.
    pub fn dependency_path(&self) -> Vec<String> {
        std::iter::once(self.declaration.to_string())
            .chain(self.path.iter().cloned())
            .collect()
    }
}

/// What `select` is given: the value, where it sits, and the relations
/// available for it.
///
/// No configuration comes with it. Which settings apply to this value is the
/// target's own lookup, over its own storage, from the type in
/// [`Crossing::ty`] and the [`Position`] — which is how a choice recorded for
/// one parameter of one function differs from the type's default.
pub struct SelectionQuery<'a> {
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
}

/// What `select` answers: how this value is read, and which conversion that
/// makes it.
///
/// The `conversion` key is the target's own name for the settings it just
/// applied. The registry never looks inside it; it compares keys, and two
/// values whose crossing, relation, children and key are all equal share one
/// conversion. So a key has one obligation, stated in #766: **equal keys mean
/// interchangeable conversions** — same layout, same operations, same
/// failures, same release. A target that minted a fresh key per visit would
/// share nothing and would defeat cycle detection; one that returned an equal
/// key for two settings that generate differently would silently give the
/// second value the first one's conversion.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Selection<K> {
    /// One of the [`SelectionQuery::candidates`], by identity.
    pub relation: RelationId,
    /// What the target decided this value converts by.
    pub conversion: K,
}

/// What `represent` is given: the selected relation, resolved against the
/// model.
///
/// No position, deliberately. A representation is reused wherever a conversion
/// with the same identity is needed, and that identity is the crossing, the
/// relation, the [`Selection::conversion`] key and the children — a target
/// that could answer differently for two positions would have its second
/// answer silently bypassed. Varying by position is done in
/// [`Target::select`], which does see one: it returns a different key, and a
/// different key is a different conversion.
pub struct ResolvedShape<'a> {
    pub crossing: &'a Crossing,
    pub relation: &'a Relation,
    /// The struct behind a struct relation, for a target that renders its own
    /// declaration of it.
    pub strukt: Option<&'a Struct>,
}

/// One already-planned child, as its parent's representation sees it.
pub struct ChildValue<'a> {
    pub part: &'a Part,
    pub layout: &'a Layout,
}

/// The planned conversions of one source function's parameters and result,
/// as the boundary and the public declaration see them.
///
/// A wrapper is these conversions around one call: each input is converted
/// from what the wrapper parameter carries, the source function is called,
/// and its result is converted for delivery. What the target reads here is
/// each plan's carrier — the layout a wrapper parameter or return must match —
/// and the failure categories the boundary has to route.
pub struct ResolvedValues<'a, P> {
    /// One plan per source parameter, in [`Function::params`] order, each
    /// crossing into Rust.
    pub inputs: Vec<&'a crate::plan::ValuePlan<P>>,
    /// The plan for the source function's result, crossing out of Rust, or
    /// `None` when it returns nothing.
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

/// The first argument of [`Target::boundary`]: which wrapper the call is about.
///
/// For every wrapper it builds, the registry calls
/// `target.boundary(&site, &values)` once, and the target returns
/// the wrapper's shape — symbol, calling convention, parameters,
/// failure routes — as a [`BoundarySpec`], wrapped in [`TargetSupport`]: a
/// spec, a reason the target cannot shape this one, or an error. `site` is
/// this type and says which source function the wrapper exports: the
/// declaration that requested the export, and the function itself. `values`
/// is the [`ResolvedValues`] the wrapper converts. What the binding asked for
/// — the exported symbol, where the public declaration lands — the target
/// looks up in its own storage, by [`SiteDescriptor::declaration`].
///
/// What a target reads from the descriptor: the declaration, for the names in
/// its refusals and errors; the source function's parameter list, for the
/// names the wrapper signature keeps; and whether a source function is there
/// at all. A descriptor with none says the wrapper is a handle's release —
/// it takes the handle back and drops it (see [`ReprSpec::release`]) — and
/// its declaration is the handle *type's*, so the target shapes it from what
/// it recorded for that type rather than for a function.
///
/// Named after the specification's *site*, a value's position in an exported
/// function: the boundary places each such position on a wrapper parameter or
/// the return, and this identifies the function the positions belong to.
pub struct SiteDescriptor<'a> {
    /// The export this wrapper is for. For a release, the handle type's own
    /// declaration — and what the target looks its configuration for this
    /// wrapper up by. The exported symbol is not spelled here; the frontend
    /// settled it when it recorded the declaration.
    pub declaration: &'a Declaration,
    /// The source function the wrapper calls once, or `None` for a release.
    ///
    /// The target reads its parameter *names*: a wrapper parameter keeps its
    /// source parameter's name, and one the target adds — a JNI environment,
    /// a receiver — is named around those. The parameter *types* come from
    /// the plans in [`ResolvedValues::inputs`], which are in
    /// [`Function::params`] order.
    pub function: Option<&'a Function>,
}

/// What `surface` is given: the requested public declaration.
///
/// [`Self::declaration`] is what the target looks its own configuration for
/// this output up by — which class or C name it was declared under, which
/// declarator it came from.
pub struct SurfaceRequest<'a> {
    pub declaration: &'a Declaration,
    pub item: SourceItem<'a>,
}

impl SurfaceRequest<'_> {
    /// The `#[cfg]` conditions the captured item behind this request was
    /// written under, spelled as the source wrote them — empty in the ordinary
    /// case.
    ///
    /// Text rather than tokens, because the registry is what puts a condition
    /// on the Rust it emits for this declaration. What is left for a target is
    /// the declaration written in *its* language, which usually cannot state a
    /// condition at all; saying so in that declaration's documentation is the
    /// most such a target can do, and is better than saying nothing.
    pub fn item_conditions(&self) -> Vec<String> {
        let conditions = match self.item {
            SourceItem::Struct(strukt) => {
                crate::emit::Writer.conditions(Conditioned::Struct(strukt))
            }
            SourceItem::Function(function) => {
                crate::emit::Writer.conditions(Conditioned::Function(function))
            }
            SourceItem::Extern(opaque) => {
                crate::emit::Writer.conditions(Conditioned::Extern(opaque))
            }
        };
        conditions
            .iter()
            .map(|condition| prebindgen_flat::close_up(&condition.to_string()))
            .collect()
    }

    /// The `#[cfg]` conditions of each field of the struct behind this request,
    /// in field order — empty entries for the ordinary unconditional field, and
    /// an empty list for a request that is not a struct.
    ///
    /// A target declaring a member per field puts its entry on that member. The
    /// registry puts the same conditions on every instruction that serves the
    /// field, so a member declared under them is read under them and the
    /// initializer that consumes the read is written under them too.
    ///
    /// Tokens rather than text, unlike [`Self::item_conditions`]: these go into
    /// the Rust a target contributes.
    pub fn field_conditions(&self) -> Vec<Vec<TokenStream>> {
        match self.item {
            SourceItem::Struct(strukt) => strukt
                .fields
                .iter()
                .map(|field| crate::emit::Writer.conditions(Conditioned::Field(field)))
                .collect(),
            SourceItem::Function(_) | SourceItem::Extern(_) => Vec::new(),
        }
    }
}

/// The captured item a public declaration is made from, as the source model
/// describes it.
///
/// A binding asks for an output by name — a [`Declaration`] such as
/// `type:Ledger` — and the registry resolves that name in the source model
/// before it plans anything. This is what it found: the model's own element,
/// borrowed from the [`Flat`] the run was planned over. A target receives it
/// as [`SurfaceRequest::item`] when it answers [`Target::surface`].
///
/// It is a view over the model and carries nothing of its own. What it adds
/// is shape: [`Element`] has variants no declaration can resolve to (a guard,
/// an unsupported item) and nests the type kinds one level down, so a target
/// matching on it would carry unreachable arms and a second match. This lists
/// exactly the kinds a resolved declaration can be — a constant and the enums
/// will add variants — so an adapter's `match` is exhaustive over real cases
/// and the compiler says when a kind is added.
///
/// The variant is a fact about the source, not about the representation: a
/// struct declared to C as an opaque pointer still arrives as
/// [`SourceItem::Struct`], and how the target carries it is in the target's
/// own configuration, under [`SurfaceRequest::declaration`]. Read that first.
/// A declaration that names no entity — a callback, a constant computed on the
/// foreign side — is refused before `surface` is asked, so nothing here is
/// ever absent. An entity the binding defined itself is here like any other:
/// a helper's stated signature arrives as [`SourceItem::Function`], an opaque
/// type declared over one the source never exported as [`SourceItem::Extern`].
///
/// [`Flat`]: prebindgen_flat::flat::Flat
/// [`Element`]: prebindgen_flat::flat::Element
#[derive(Clone, Copy)]
pub enum SourceItem<'a> {
    /// A captured free function, the item behind a `fn:` declaration.
    ///
    /// The target reads its signature — [`Function::params`] and
    /// [`Function::ret`] — to spell the foreign declaration, and pairs each
    /// parameter with the already-planned conversion the registry hands over
    /// in [`ResolvedValues::inputs`], in the same order. The call itself is
    /// not the target's business: the registry places it when it assembles
    /// the wrapper, from the [`SiteDescriptor`] it asked [`Target::boundary`]
    /// about.
    Function(&'a Function),
    /// A captured struct with fields the model can inspect, the usual item
    /// behind a `type:` declaration.
    ///
    /// A target declaring one — a `repr(C)` mirror, a Kotlin `data class` —
    /// re-declares its fields: [`Struct::fields`], with the `#[cfg]`
    /// condition of each in [`SurfaceRequest::field_conditions`]. A struct
    /// the binding declared to be carried whole crosses as a handle instead,
    /// and its fields go unread; the target learns that from its own
    /// configuration, not from this variant. A tuple struct is not this
    /// variant at all: its fields are not modelled, and the model declares it
    /// as [`SourceItem::Extern`].
    Struct(&'a Struct),
    /// A captured declaration with nothing behind it — a marked type alias
    /// such as `pub type Ledger = crate::ledger::Ledger;`, or a tuple struct —
    /// which the source model calls an extern.
    ///
    /// It has no parts to convert through, so a value of it crosses whole: as
    /// an opaque handle, represented by a carrier for the address and a
    /// release for it (see [`ReprSpec::release`]). [`Extern::target`]
    /// says what the alias pointed at, as text, for a target that wants to
    /// recognise one; the model does not classify it. A target with no handle
    /// representation refuses this variant, and one that declared this type to
    /// be read through its fields finds no struct relation to select at all.
    Extern(&'a Extern),
}

// ---------------------------------------------------------------------------
// The interface
// ---------------------------------------------------------------------------

/// The language adapter, as the engine sees it.
///
/// Every method answers a local question about one value, one call or one
/// declaration. None of them walks a type, allocates a name, or decides control
/// flow: those are the registry's, which is why a second field, a nested struct
/// or a third target costs an adapter nothing new.
///
/// # Where the settings live
///
/// Here. The registry stores no configuration of its own and knows no
/// precedence rule: it does not hold a table of the binding's choices, and it
/// cannot say whether one recorded for a parameter outranks one recorded for a
/// type. An adapter keeps its `build.rs` storage — the same storage its v1
/// route reads — and answers each question below out of it, addressed by
/// [`SelectionQuery::position`] for a value and by the [`Declaration`] for an
/// output.
///
/// What the registry does own is *identity*: [`Target::select`] returns a
/// [`Selection`] whose `conversion` key stands for the settings the adapter
/// just applied, and the registry reuses one conversion for every value whose
/// crossing, relation, children and key all match. Hence the one rule an
/// adapter owes it, [`Selection`]'s: equal keys are interchangeable
/// conversions, and settings that generate differently get different keys.
/// A lookup that silently fell back to a default where the binding asked for
/// something the adapter cannot do yet would break it in the other direction —
/// the answer must be a reported [`Unsupported`], not the default.
pub trait Target {
    /// This target's name in a report — `"c"`, `"jni"`. Intrinsic to the
    /// adapter, so nothing has to carry it alongside the requests.
    const NAME: &'static str;

    /// What the adapter calls one way of converting a value.
    ///
    /// The registry never looks inside it: it compares keys to decide which
    /// values share a conversion ([`Selection`]), and to notice a type whose
    /// conversion needs its own. The obligations that come with minting one
    /// are [`Selection`]'s.
    ///
    /// An adapter whose settings are plain data can use those directly; one
    /// holding something incomparable — a naming closure, say, which cannot be
    /// compared to another closure — interns it and uses the index.
    type ConversionKey: Clone + Eq + std::hash::Hash;
    /// The adapter's own rendering data, retained in the plans and read back by
    /// [`Target::render_operation`] and by the adapter's foreign writer.
    ///
    /// `Clone` because one described operation is applied wherever it is
    /// needed: the description is registered once per use site, and the
    /// alternative is an adapter allocating its own identities for the registry
    /// to trust.
    type Payload: Clone;

    /// Choose the relation this value is read through, and name the conversion
    /// that makes it.
    ///
    /// This is the one call that sees a [`Position`], and so the only place a
    /// per-site choice can take effect: it does so by coming back as a
    /// different [`Selection::conversion`].
    fn select(&self, query: &SelectionQuery<'_>) -> TargetSupport<Selection<Self::ConversionKey>>;

    /// Describe what carries this value and how its parts are accessed, for
    /// the conversion [`Target::select`] named.
    fn represent(
        &self,
        shape: &ResolvedShape<'_>,
        children: &[ChildValue<'_>],
        conversion: &Self::ConversionKey,
    ) -> TargetSupport<ReprSpec<Self::Payload>>;

    /// Describe the wrapper that will export this source function: its
    /// interface, and its routes for the failures its conversions can raise.
    ///
    /// What the binding asked for this export — its symbol above all — the
    /// adapter reads from its own storage under [`SiteDescriptor::declaration`].
    fn boundary(
        &self,
        site: &SiteDescriptor<'_>,
        values: &ResolvedValues<'_, Self::Payload>,
    ) -> TargetSupport<BoundarySpec<Self::Payload>>;

    /// Describe one public declaration and what it requires, from what the
    /// binding recorded under [`SurfaceRequest::declaration`].
    fn surface(
        &self,
        request: &SurfaceRequest<'_>,
        values: &ResolvedValues<'_, Self::Payload>,
    ) -> TargetSupport<SurfaceSpec<Self::Payload>>;

    /// Render one of this target's operations as a single Rust expression.
    ///
    /// One expression and nothing around it: no `let`, no `match`, no return.
    /// The operands arrive already named by the writer, in the order the
    /// operation's specification lists them.
    fn render_operation(&self, payload: &Self::Payload, operands: &[syn::Ident]) -> TokenStream;

    /// Say, for the report, what this declaration is: the adapter's own
    /// declarator word and where the thing lands in the foreign language.
    ///
    /// Read from the same storage [`Target::boundary`] and [`Target::surface`]
    /// read, under the same [`Declaration`], so the report cannot say one
    /// thing and the generated code another. Asked of every requested output,
    /// including one that was skipped — the report says what a declaration was
    /// *for*, not only what became of it.
    fn describe(&self, declaration: &Declaration) -> Described;
}

/// How a report names one declaration on the foreign side — see
/// [`Target::describe`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Described {
    /// The declarator that produced it (`opaque_ptr`, `data_class`, `fun`, …)
    /// — the adapter's own word, printed back verbatim. Finer than the prefix
    /// a [`Declaration`] prints with: `opaque_ptr` and `data_struct` are both
    /// a `type`.
    pub representation: String,
    /// Where it lands in the target language, spelled the way that language
    /// spells it: `calculator_t`, `io.zenoh.jni.Session`.
    pub placement: String,
}

impl Described {
    pub fn new(representation: impl Into<String>, placement: impl Into<String>) -> Self {
        Described {
            representation: representation.into(),
            placement: placement.into(),
        }
    }
}
