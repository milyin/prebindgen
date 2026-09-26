//! What a frontend states before planning: the whole of what planning reads.
//!
//! A frontend turns its user's configuration into one [`Binding`] — the
//! wire types generated Rust may hold a value in, the representations values
//! cross as, the rules saying which values take which representation, and the
//! outputs to expose — and hands it to [`generate`](crate::generate) with its
//! [`Target`]. The registry plans from the binding and the source model alone:
//! no target code runs until the plan is complete. So the configuration can be
//! checked before planning, printed, and diffed.

use prebindgen_flat::TypeKey;

use crate::{
    decl::Declaration,
    target::{Direction, FailureCategory, Target, Terminal, Unsupported},
};

/// Which binding issued an id. Every [`Binding`] draws a fresh one, and each id
/// it issues carries it, so an id handed to another binding is recognised as
/// foreign instead of naming whatever sits at the same index there.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Issuer(u64);

impl Issuer {
    fn fresh() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Issuer(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

/// A wire type the binding declared, valid only in the binding that issued it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WireTypeId {
    issuer: Issuer,
    pub(crate) index: usize,
}

/// An into-Rust representation the binding declared, valid only in the
/// binding that issued it. Equal representations get one id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InReprId {
    issuer: Issuer,
    pub(crate) index: usize,
}

/// An out-of-Rust representation the binding declared, valid only in the
/// binding that issued it. Equal representations get one id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutReprId {
    issuer: Issuer,
    pub(crate) index: usize,
}

/// A representation of either direction: what a rule names, and a
/// conversion's identity. Its direction is the direction of every value it
/// serves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReprId {
    In(InReprId),
    Out(OutReprId),
}

impl From<InReprId> for ReprId {
    fn from(id: InReprId) -> Self {
        ReprId::In(id)
    }
}

impl From<OutReprId> for ReprId {
    fn from(id: OutReprId) -> Self {
        ReprId::Out(id)
    }
}

impl ReprId {
    fn issuer(self) -> Issuer {
        match self {
            ReprId::In(id) => id.issuer,
            ReprId::Out(id) => id.issuer,
        }
    }

    /// The direction of the values it serves.
    pub fn direction(self) -> Direction {
        match self {
            ReprId::In(_) => Direction::IntoRust,
            ReprId::Out(_) => Direction::OutOfRust,
        }
    }
}

/// One of the binding's outputs, valid only in the binding that issued it.
///
/// An output is a declaration *and* the form recorded with it, and the pair is
/// what tells two of them apart: one entity declared twice — `Stamp` as a data
/// class and as a handle — is two outputs, each planned, accounted for and
/// required separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputId {
    issuer: Issuer,
    pub(crate) index: usize,
}

impl std::fmt::Display for WireTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "w{}", self.index)
    }
}

impl std::fmt::Display for InReprId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "in{}", self.index)
    }
}

impl std::fmt::Display for OutReprId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "out{}", self.index)
    }
}

impl std::fmt::Display for ReprId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReprId::In(id) => id.fmt(f),
            ReprId::Out(id) => id.fmt(f),
        }
    }
}

/// What kind of wire type a value crosses in: C's `I64`, `Pointer`,
/// `Aggregate`; JNI's `Long`, `Int`, `Handle`, `Object`.
///
/// A kind carries no data, so what a target can do with a wire type — which
/// parts it can have, what a wrapper can take or return — is stated per kind
/// and cannot depend on any one wire type's name or metadata.
pub trait WireKind: Copy + Eq + std::hash::Hash + std::fmt::Debug + 'static {
    /// Every kind of the target.
    const ALL: &'static [Self];

    /// How a refusal names it: `pointer`, in `unsupported.c.member.pointer`.
    ///
    /// It is part of a capability code, which a build script may match and a
    /// report groups skips by, so it is the kind's identity in text: unique
    /// among [`ALL`](Self::ALL) — [`generate`](crate::generate) refuses a
    /// target with two kinds of one name — and kept stable when the target
    /// changes, as a capability code is.
    fn name(self) -> &'static str;

    /// The kinds a wire type of this kind can have as its parts — a C struct's
    /// members, a JVM object's properties, a callback's arguments. Empty for a
    /// kind that has no parts.
    fn parts(self) -> &'static [Self] {
        &[]
    }
}

/// The type of a value on the boundary, as both sides see it: its Rust type,
/// and what the foreign side reads it as. `i64` and a Kotlin `Long`; `*mut
/// Ledger` and the C handle `Ledger *`; a `JObject` and an `example.Stamp`.
///
/// A target's wire types are its own enum, one variant per kind, each holding
/// the data only that kind needs: C's name for a struct it declares, a Kotlin
/// class's name. Two wire types of one Rust type differ when the foreign side
/// reads them differently — a `jlong` holding a number and one holding an
/// address — so equality is the whole value's.
pub trait WireType: Clone + Eq + std::hash::Hash + std::fmt::Debug {
    type Kind: WireKind;

    fn kind(&self) -> Self::Kind;

    /// The Rust type: `i64`, `*mut Ledger`, `Stamp`, `jni::objects::JObject<'_>`.
    fn rust(&self) -> syn::Type;
}

/// The kind of a target's wire types.
pub type WireKindOf<T> = <<T as Target>::WireType as WireType>::Kind;

/// How the values a rule covers cross into Rust.
///
/// It names its wire types and operations and states no type for either: a
/// whole value's operation reads its wire type and produces the source type, a
/// `read` reads the parts' wire type and produces the part's, whatever wire
/// type that part resolves to. The registry works them out when it plans a value, and feeds
/// them to the writer.
// A binding holds a few dozen of these, each once, so the unboxed variants
// cost nothing worth an indirection.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InRepresentation<Op> {
    /// The whole value, one operation: a scalar, a handle, a fieldless enum.
    Whole {
        wire_type: WireTypeId,
        /// From the wire type to the source type.
        operation: Operation<Op>,
    },
    /// The parts of a relation, carried together in one wire type.
    Parts {
        via: Via,
        /// The aggregate or object holding the parts: a wire type of a kind
        /// that has parts.
        wire_type: WireTypeId,
        /// One part out of the wire type, applied once per part.
        read: Operation<Op>,
    },
    /// A foreign callable, as an `impl Fn(..)`: the registry builds the
    /// closure, whose every call converts the arguments out of Rust and hands
    /// them to `invoke`.
    Callable {
        /// What holds the callable on the foreign side: a C closure struct, a
        /// JVM object. Its parts are the arguments' wire types.
        wire_type: WireTypeId,
        /// Applied once to the wire type, where the callable enters Rust: what
        /// every call will need, such as a reference the JVM keeps alive. Its
        /// result is moved into the closure.
        capture: Operation<Op>,
        /// Applied on every call to what `capture` produced and the
        /// arguments' wire types, in order. It produces nothing.
        invoke: Operation<Op>,
        /// What a call does when converting an argument or invoking fails:
        /// the closure returns nothing, so no caller can take the failure.
        /// Reporting inside a call has no runtime context to use.
        routes: Vec<FailureRoute<Op>>,
    },
    /// A representation the target does not lower, refused by name.
    Unsupported(Unsupported),
}

/// How the values a rule covers cross out of Rust.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OutRepresentation<Op> {
    /// The whole value, one operation: a scalar, a handle, a fieldless enum.
    Whole {
        wire_type: WireTypeId,
        /// From the source type to the wire type.
        operation: Operation<Op>,
        /// Frees what `operation` handed out, taking it in the wire type: a
        /// handle's typed drop. Its presence is what makes the value a
        /// handle; a type output exposing one exports it as a wrapper of its
        /// own.
        release: Option<Operation<Op>>,
    },
    /// A representation the target does not lower, refused by name.
    Unsupported(Unsupported),
}

impl<Op> OutRepresentation<Op> {
    /// What a type read through its fields into Rust states out of Rust: the
    /// registry cannot build a value from its parts yet, so every target
    /// refuses such a value leaving Rust under this one code.
    pub fn struct_unsupported(ty: &TypeKey) -> Self {
        OutRepresentation::Unsupported(Unsupported::new(
            "unsupported.struct.out_of_rust",
            format!(
                "`{}` leaves Rust as a composed value; v2 has no target construction \
                 operation yet",
                ty.as_str()
            ),
        ))
    }
}

/// Which relation of its type [`InRepresentation::Parts`] reads a value through.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Via {
    /// The struct relation: one part per field.
    Fields,
}

/// One operation, as the binding states it: which one, and nothing else.
///
/// What it needs and what it can raise are facts of the operation itself —
/// the registry's own table for a standard one, the target's
/// [`TargetOp`] for one of its own — so no use of an operation restates them
/// and no two uses can disagree.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Operation<Op> {
    /// One the registry writes itself.
    Standard(StandardOp),
    /// One of the target's own, which its writer writes.
    Target(Op),
}

impl<Op: TargetOp> Operation<Op> {
    /// The runtime contexts it needs besides its operand, by name:
    /// `"jni.env"`. A standard operation needs none.
    pub fn contexts(&self) -> &'static [&'static str] {
        match self {
            Operation::Standard(_) => &[],
            Operation::Target(op) => op.contexts(),
        }
    }

    /// Its failure category and error type, if it can fail.
    pub fn failure(&self) -> Option<Failure> {
        match self {
            Operation::Standard(op) => op.failure(),
            Operation::Target(op) => op.failure(),
        }
    }
}

/// An operation only a target's language has — a JVM getter call, a throw —
/// and the two facts about it the registry plans with. Both are the
/// operation's own, the same wherever it is applied.
pub trait TargetOp: Clone + Eq + std::hash::Hash + std::fmt::Debug {
    /// The runtime contexts it needs besides its operand, by name: `"jni.env"`.
    /// A function form's context parameters supply them, and nothing supplies
    /// one inside a callback's call.
    fn contexts(&self) -> &'static [&'static str] {
        &[]
    }

    /// Its failure category and error type, if it can fail.
    fn failure(&self) -> Option<Failure> {
        None
    }
}

/// How an operation fails.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Failure {
    pub category: FailureCategory,
    pub error: Box<syn::Type>,
}

impl Failure {
    /// A binding failure carrying a `String` message, which is what every
    /// standard operation that can fail raises.
    pub fn binding_message() -> Self {
        Failure {
            category: FailureCategory::Binding,
            error: Box::new(syn::parse_quote!(String)),
        }
    }
}

/// An operation the registry writes itself, because writing it means naming a
/// source type, which only the registry can do.
///
/// None states a type: where it is used says which. What differs per target —
/// the wire type an address is cast to, what a value is carried as — is the
/// wire type's and the arms'.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StandardOp {
    /// The result *is* the operand: the wire type and the source value are one
    /// Rust value. Renders nothing at all.
    Identity,
    /// Read the part's member of the aggregate the wire type is: `arg0.secs`.
    ReadMember,
    /// Hand an owned source value to the foreign side as an address:
    /// `Box::into_raw(Box::new(v)) as <wire type>`. The foreign side owns the
    /// allocation from here on.
    IntoRaw,
    /// Take back what [`StandardOp::IntoRaw`] handed out:
    /// `*Box::from_raw(v as *mut <source>)`. A null address is a binding
    /// failure carrying a `String`; the handle is consumed either way.
    FromRaw,
    /// Drop what [`StandardOp::IntoRaw`] handed out without converting it. A
    /// null address releases nothing, as `free(NULL)` does. Produces no value.
    Release,
    /// Take a value of a fieldless source enum to what carries it: `match v {
    /// source::Op::Add => <carried>, … }`, one arm per value.
    EnumOut { values: Vec<crate::target::EnumArm> },
    /// The reverse: a carried value back to the source enum.
    ///
    /// With `invalid` set the match ends in a default arm that fails with
    /// that message, formatted with the value — the wire type can hold
    /// something no value of the enum names. With `bits` set the match is on
    /// the wire type's bits read as that integer type: C's `MaybeUninit<op_t>`
    /// holds whatever `int` the caller passed. The wire type must be exactly as
    /// large as `bits`, and initialized, which only the caller can promise, so
    /// the registry makes a wrapper that reads one `unsafe`.
    EnumIn {
        values: Vec<crate::target::EnumArm>,
        invalid: Option<String>,
        bits: Option<Box<syn::Type>>,
    },
}

impl StandardOp {
    /// The failure a standard operation has by definition: taking back a null
    /// handle, and reading a number no value of an enum has where the enum
    /// says how to refuse one — each a binding failure carrying a message.
    pub fn failure(&self) -> Option<Failure> {
        match self {
            StandardOp::FromRaw
            | StandardOp::EnumIn {
                invalid: Some(_), ..
            } => Some(Failure::binding_message()),
            _ => None,
        }
    }
}

/// How one source function is exported: everything about its wrapper that is
/// not a planned value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionForm<Op> {
    /// The `extern` string: `"C"`, `"system"`.
    pub abi: String,
    /// The symbol the wrapper is exported under.
    pub symbol: String,
    /// Wrapper parameters no source parameter feeds, before those that do:
    /// the JNI environment, the receiver.
    pub context: Vec<ContextParam>,
    /// The names of the wrapper parameters that carry the inputs, in source
    /// order — the source's own names for a function, one name for a
    /// release.
    pub inputs: Vec<syn::Ident>,
    /// One route per failure category the conversions can raise. A category
    /// with no route makes the export unsupported.
    pub routes: Vec<FailureRoute<Op>>,
    /// Attributes beyond `#[no_mangle]`, which the writer owns.
    pub attrs: Vec<syn::Attribute>,
    /// Whether the wrapper is an `unsafe fn`.
    pub unsafety: bool,
}

/// A wrapper parameter the calling convention adds.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ContextParam {
    pub name: syn::Ident,
    pub ty: syn::Type,
    /// The runtime context it supplies, by the name operations ask for it
    /// under; `None` for one the convention requires and nothing uses.
    pub supplies: Option<String>,
    /// Whether the binding needs `mut`.
    pub mutable: bool,
}

/// What a wrapper does when a conversion fails in one category.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FailureRoute<Op> {
    pub category: FailureCategory,
    /// An operation reporting the error before the call ends, handed the
    /// error of the type stated here.
    pub report: Option<Report<Op>>,
    /// What to do when reporting itself fails.
    pub on_report_failure: Terminal,
    /// How this route ends.
    pub terminate: Terminal,
}

/// A failure route's reporting operation, and the error type it takes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Report<Op> {
    pub error: syn::Type,
    pub operation: Operation<Op>,
}

/// What one output is.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum OutputForm<Op, O> {
    /// A type's representations, exposed: their wire types' Rust
    /// declarations, and the type's foreign declaration. Each representation
    /// is also the rule at the output's root in its direction, so two outputs
    /// of one type each plan as declared.
    Type {
        /// How the type crosses into Rust. Every type output has one: even a
        /// handle handed out has to come back.
        into_rust: InReprId,
        /// How it crosses out of Rust, if it does.
        out_of_rust: Option<OutReprId>,
        /// The wrapper releasing a handed-out value, for an out-of-Rust
        /// representation that has a release.
        release: Option<FunctionForm<Op>>,
        meta: O,
    },
    /// A source function, exported through a wrapper of this form.
    Function { form: FunctionForm<Op>, meta: O },
    /// A declarator the target does not lower, refused by name.
    Unsupported(Unsupported),
}

/// Which values a conversion rule covers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Scope {
    /// Every value of this type, wherever it turns up.
    Type(TypeKey),
    /// The one value at this path inside this output.
    At(OutputId, ValuePath),
}

/// From an output's root to one value inside it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ValuePath(pub Vec<Step>);

impl ValuePath {
    pub fn root() -> Self {
        ValuePath(Vec::new())
    }

    /// This path, one step further.
    pub fn then(&self, step: Step) -> Self {
        let mut steps = self.0.clone();
        steps.push(step);
        ValuePath(steps)
    }
}

impl std::fmt::Display for ValuePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let steps: Vec<String> = self.0.iter().map(Step::to_string).collect();
        write!(f, "{}", steps.join("."))
    }
}

/// One step of a [`ValuePath`], named as the source names it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Step {
    /// A source function's parameter, by the name the source gives it.
    Param(String),
    /// A source function's result.
    Return,
    /// A field of the struct the current value is read through: its name, or
    /// its position for a tuple field.
    Field(String),
    /// An argument of the callback the current value is, by position: the
    /// value Rust hands the foreign callable, out of Rust.
    Arg(usize),
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Param(name) => write!(f, "param {name}"),
            Step::Return => write!(f, "return"),
            Step::Field(name) => write!(f, "field {name}"),
            Step::Arg(index) => write!(f, "arg {index}"),
        }
    }
}

/// An into-Rust representation, as a binding for target `T` states one.
pub type InRepresentationOf<T> = InRepresentation<<T as Target>::Op>;
/// An out-of-Rust representation, as a binding for target `T` states one.
pub type OutRepresentationOf<T> = OutRepresentation<<T as Target>::Op>;
/// A function form, as a binding for target `T` states one.
pub type FunctionFormOf<T> = FunctionForm<<T as Target>::Op>;
/// An output's form, as a binding for target `T` states one.
pub type OutputFormOf<T> = OutputForm<<T as Target>::Op, <T as Target>::OutputMeta>;

/// Everything planning reads from a binding.
///
/// The ids it issues are valid only in it. Every method taking an id panics if
/// the id was issued by another binding: that is a frontend bug, found where
/// the id is handed over rather than as a wrong plan.
pub struct Binding<T: Target> {
    issuer: Issuer,
    wire_types: Vec<T::WireType>,
    in_representations: Vec<InRepresentationOf<T>>,
    out_representations: Vec<OutRepresentationOf<T>>,
    rules: Vec<(Scope, ReprId)>,
    outputs: Vec<(Declaration, OutputFormOf<T>)>,
}

impl<T: Target> Default for Binding<T> {
    fn default() -> Self {
        Binding {
            issuer: Issuer::fresh(),
            wire_types: Vec::new(),
            in_representations: Vec::new(),
            out_representations: Vec::new(),
            rules: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

impl<T: Target> Binding<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// A wire type generated Rust may use. Declaring an equal wire type again
    /// returns the same id.
    pub fn wire_type(&mut self, wire_type: T::WireType) -> WireTypeId {
        WireTypeId {
            issuer: self.issuer,
            index: intern(&mut self.wire_types, wire_type),
        }
    }

    /// One way a type crosses into Rust. Declaring an equal representation
    /// again returns the same id.
    ///
    /// Panics if a wire type it names was issued by another binding.
    pub fn in_representation(&mut self, representation: InRepresentationOf<T>) -> InReprId {
        match &representation {
            InRepresentation::Whole { wire_type, .. }
            | InRepresentation::Parts { wire_type, .. }
            | InRepresentation::Callable { wire_type, .. } => {
                self.check(wire_type.issuer, "wire type")
            }
            InRepresentation::Unsupported(_) => {}
        }
        InReprId {
            issuer: self.issuer,
            index: intern(&mut self.in_representations, representation),
        }
    }

    /// One way a type crosses out of Rust. Declaring an equal representation
    /// again returns the same id.
    ///
    /// Panics if a wire type it names was issued by another binding.
    pub fn out_representation(&mut self, representation: OutRepresentationOf<T>) -> OutReprId {
        if let OutRepresentation::Whole { wire_type, .. } = &representation {
            self.check(wire_type.issuer, "wire type");
        }
        OutReprId {
            issuer: self.issuer,
            index: intern(&mut self.out_representations, representation),
        }
    }

    /// The values `scope` covers cross as `representation`, in its direction.
    ///
    /// Panics if `representation`, or the output `scope` names, was issued by
    /// another binding.
    pub fn rule(&mut self, scope: Scope, representation: impl Into<ReprId>) {
        let representation = representation.into();
        self.check(representation.issuer(), "representation");
        if let Scope::At(output, _) = &scope {
            self.check(output.issuer, "output");
        }
        self.rules.push((scope, representation));
    }

    /// Expose `declaration` in `form`. The id is how a rule addresses a value
    /// inside the output.
    ///
    /// Panics if a representation a type form names was issued by another
    /// binding.
    pub fn output(&mut self, declaration: Declaration, form: OutputFormOf<T>) -> OutputId {
        if let OutputForm::Type {
            into_rust,
            out_of_rust,
            ..
        } = &form
        {
            self.check(into_rust.issuer, "representation");
            if let Some(out_of_rust) = out_of_rust {
                self.check(out_of_rust.issuer, "representation");
            }
        }
        self.outputs.push((declaration, form));
        self.output_id(self.outputs.len() - 1)
    }

    /// The wire type `id` names. Panics if another binding issued `id`.
    pub fn wire_type_of(&self, id: WireTypeId) -> &T::WireType {
        self.check(id.issuer, "wire type");
        &self.wire_types[id.index]
    }

    /// The into-Rust representation `id` names. Panics if another binding
    /// issued `id`.
    pub fn in_representation_of(&self, id: InReprId) -> &InRepresentationOf<T> {
        self.check(id.issuer, "representation");
        &self.in_representations[id.index]
    }

    /// The out-of-Rust representation `id` names. Panics if another binding
    /// issued `id`.
    pub fn out_representation_of(&self, id: OutReprId) -> &OutRepresentationOf<T> {
        self.check(id.issuer, "representation");
        &self.out_representations[id.index]
    }

    /// Every rule, in the order the frontend stated them.
    pub fn rules(&self) -> &[(Scope, ReprId)] {
        &self.rules
    }

    /// Every output, in the order the frontend stated them; an output's
    /// position here is its id's.
    pub fn outputs(&self) -> &[(Declaration, OutputFormOf<T>)] {
        &self.outputs
    }

    /// The declaration and form recorded as one output. Panics if another
    /// binding issued `id`.
    pub fn output_of(&self, id: OutputId) -> &(Declaration, OutputFormOf<T>) {
        self.check(id.issuer, "output");
        &self.outputs[id.index]
    }

    /// The form recorded with one output. Panics if another binding issued
    /// `id`.
    pub fn form_of(&self, id: OutputId) -> &OutputFormOf<T> {
        &self.output_of(id).1
    }

    /// The id of the output at `index` in [`outputs`](Self::outputs).
    pub(crate) fn output_id(&self, index: usize) -> OutputId {
        OutputId {
            issuer: self.issuer,
            index,
        }
    }

    /// An id this binding issued exists in it, since nothing is ever removed:
    /// the issuer is all there is to check.
    fn check(&self, issuer: Issuer, kind: &str) {
        assert!(
            issuer == self.issuer,
            "a {kind} id issued by another binding was handed to this one"
        );
    }
}

impl<T: Target> std::fmt::Display for Binding<T> {
    /// One line per wire type, representation, rule and output, a function
    /// form's details on the lines under its output, with the target's own
    /// vocabulary in its `Debug` form. It prints every field planning reads,
    /// which makes it the diagnostic to diff when two builds of one binding
    /// generate differently. It is not a proof that two bindings are equal:
    /// the target's wire types and operations print only as much as their
    /// `Debug` tells apart.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, wire_type) in self.wire_types.iter().enumerate() {
            writeln!(
                f,
                "wire     w{index}  {}  {}  {:?}",
                tokens(&wire_type.rust()),
                wire_type.kind().name(),
                wire_type
            )?;
        }
        for (index, representation) in self.in_representations.iter().enumerate() {
            let described = match representation {
                InRepresentation::Whole {
                    wire_type,
                    operation: whole,
                } => format!("whole  {wire_type} {}", operation(whole)),
                InRepresentation::Parts {
                    via,
                    wire_type,
                    read,
                } => {
                    format!("parts  {wire_type}  {via:?}  read: {}", operation(read))
                }
                InRepresentation::Callable {
                    wire_type,
                    capture,
                    invoke,
                    routes,
                } => format!(
                    "callable  {wire_type}  capture: {}  invoke: {}  routes: [{}]",
                    operation(capture),
                    operation(invoke),
                    routes.iter().map(route).collect::<Vec<_>>().join("; ")
                ),
                InRepresentation::Unsupported(reason) => unsupported(reason),
            };
            writeln!(f, "repr     in{index}  {described}")?;
        }
        for (index, representation) in self.out_representations.iter().enumerate() {
            let described = match representation {
                OutRepresentation::Whole {
                    wire_type,
                    operation: whole,
                    release,
                } => format!(
                    "whole  {wire_type} {}{}",
                    operation(whole),
                    match release {
                        Some(release) => format!("  release: {}", operation(release)),
                        None => String::new(),
                    }
                ),
                OutRepresentation::Unsupported(reason) => unsupported(reason),
            };
            writeln!(f, "repr     out{index}  {described}")?;
        }
        for (scope, representation) in &self.rules {
            let scope = match scope {
                Scope::Type(key) => format!("type {}", key.as_str()),
                Scope::At(output, path) => {
                    format!("at {} {path}", self.outputs[output.index].0)
                }
            };
            writeln!(f, "rule     {scope}  {representation}")?;
        }
        for (declaration, form) in &self.outputs {
            match form {
                OutputForm::Type {
                    into_rust,
                    out_of_rust,
                    release,
                    meta,
                } => {
                    let out_of_rust = match out_of_rust {
                        Some(id) => id.to_string(),
                        None => "-".to_string(),
                    };
                    writeln!(
                        f,
                        "output   {declaration}  type {into_rust} {out_of_rust}  {meta:?}"
                    )?;
                    if let Some(release) = release {
                        write_form(f, "release", release)?;
                    }
                }
                OutputForm::Function { form, meta } => {
                    writeln!(f, "output   {declaration}  function  {meta:?}")?;
                    write_form(f, "form", form)?;
                }
                OutputForm::Unsupported(reason) => writeln!(
                    f,
                    "output   {declaration}  unsupported {}: {}",
                    reason.capability, reason.explanation
                )?,
            }
        }
        Ok(())
    }
}

/// Declaring an equal value again returns the index of the first.
fn intern<V: PartialEq>(known: &mut Vec<V>, value: V) -> usize {
    match known.iter().position(|one| *one == value) {
        Some(index) => index,
        None => {
            known.push(value);
            known.len() - 1
        }
    }
}

fn unsupported(reason: &Unsupported) -> String {
    format!("unsupported  {}: {}", reason.capability, reason.explanation)
}

fn tokens(item: &impl quote::ToTokens) -> String {
    item.to_token_stream().to_string()
}

/// An operation on one line: what it is, the contexts it needs and the failure
/// it can raise.
fn operation<Op: TargetOp>(operation: &Operation<Op>) -> String {
    let mut text = format!("{operation:?}");
    if !operation.contexts().is_empty() {
        text += &format!(" needs {}", operation.contexts().join(", "));
    }
    if let Some(failure) = &operation.failure() {
        text += &format!(
            " fails {} {}",
            failure.category.as_str(),
            tokens(&*failure.error)
        );
    }
    text
}

fn terminal(terminal: &Terminal) -> String {
    match terminal {
        Terminal::Return(value) => format!("return {}", tokens(value)),
        Terminal::Abort => "abort".to_string(),
    }
}

/// A function form, indented under the output it belongs to: its signature on
/// one line, then one line per route.
fn write_form<Op: TargetOp>(
    f: &mut std::fmt::Formatter<'_>,
    label: &str,
    form: &FunctionForm<Op>,
) -> std::fmt::Result {
    let context: Vec<String> = form
        .context
        .iter()
        .map(|param| {
            format!(
                "{}{}: {}{}",
                if param.mutable { "mut " } else { "" },
                param.name,
                tokens(&param.ty),
                match &param.supplies {
                    Some(name) => format!(" supplies {name}"),
                    None => String::new(),
                }
            )
        })
        .collect();
    let inputs: Vec<String> = form.inputs.iter().map(ToString::to_string).collect();
    let attrs: Vec<String> = form.attrs.iter().map(tokens).collect();
    writeln!(
        f,
        "         {label}  {}extern {:?} {}  context [{}]  inputs [{}]{}",
        if form.unsafety { "unsafe " } else { "" },
        form.abi,
        form.symbol,
        context.join(", "),
        inputs.join(", "),
        match attrs.is_empty() {
            true => String::new(),
            false => format!("  attrs [{}]", attrs.join(" ")),
        }
    )?;
    for one in &form.routes {
        writeln!(f, "         {label}  route {}", route(one))?;
    }
    Ok(())
}

/// A failure route on one line: its category, what reports it, and how it
/// ends.
fn route<Op: TargetOp>(route: &FailureRoute<Op>) -> String {
    let report = match &route.report {
        Some(report) => format!(
            "report {} by {}, if that fails {}, then ",
            tokens(&report.error),
            operation(&report.operation),
            terminal(&route.on_report_failure)
        ),
        None => String::new(),
    };
    format!(
        "{}: {report}{}",
        route.category.as_str(),
        terminal(&route.terminate)
    )
}
