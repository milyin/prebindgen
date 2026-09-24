//! What a frontend states before planning: the whole of what planning reads.
//!
//! A frontend turns its user's configuration into one [`Binding`] — the
//! carriers generated Rust may hold a value in, the representations values
//! cross as, the rules saying which values take which representation, and the
//! outputs to expose — and hands it to [`generate`](crate::generate) with its
//! [`Target`]. The registry plans from the binding and the source model alone:
//! no target code runs until the plan is complete. So the configuration can be
//! checked before planning, printed, and diffed.

use prebindgen_flat::TypeKey;
use syn::visit_mut::VisitMut as _;

use crate::{
    decl::Declaration,
    target::{FailureCategory, Target, Terminal, Unsupported},
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

/// A carrier the binding declared, valid only in the binding that issued it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CarrierId {
    issuer: Issuer,
    pub(crate) index: usize,
}

/// A representation the binding declared, valid only in the binding that
/// issued it. Equal representations get one id, which is what makes it a
/// conversion's identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReprId {
    issuer: Issuer,
    pub(crate) index: usize,
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

impl std::fmt::Display for CarrierId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c{}", self.index)
    }
}

impl std::fmt::Display for ReprId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.index)
    }
}

/// One of a target's few wire types: a kind of Rust type a value may cross
/// the boundary in, which is what everything that holds a value — an
/// aggregate's members, a wrapper's parameters — states its acceptance in.
///
/// Every target states the same three things about each of its classes, and
/// the registry reads them the same way for every target.
pub trait WireClass: Clone + Eq + std::hash::Hash + std::fmt::Debug {
    /// Every class of the target: what a holder accepting anything accepts.
    fn all() -> Vec<Self>;

    /// How a refusal names it: `pointer`, in `unsupported.c.member.pointer`.
    ///
    /// It is part of a capability code, which a build script may match and a
    /// report groups skips by, so it is the class's identity in text: unique
    /// among the classes [`all`](Self::all) returns — [`generate`](crate::generate)
    /// refuses a target with two classes of one name — and kept stable when the
    /// target changes, as a capability code is.
    fn name(&self) -> &'static str;

    /// The Rust type a carrier of this class is. A class naming one exact
    /// type is written as that type — `i64`, `jni::sys::jlong`. A class whose
    /// carriers are types the target declares itself writes `_` where the
    /// declared type's name goes: `_` for a `repr(C)` struct, `*mut _` for a
    /// pointer to an incomplete one. [`WireType::declared`] fills it in.
    fn rust(&self) -> syn::Type;
}

/// A Rust type generated code may hold a value in at the boundary.
///
/// Its Rust type is its class's, so a carrier and its class cannot disagree:
/// it is built with [`WireType::exact`] for a class naming one type, and with
/// [`WireType::declared`], given the declared type's name, for one that
/// leaves the name to the target.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WireType<C, M> {
    rust: syn::Type,
    /// Which of the adapter's few wire types it is — what a holder's
    /// [`Accepts`] is stated in.
    pub class: C,
    /// For an aggregate, which wire types its members may be; `None` for a
    /// carrier with no members. A `Product` needs a carrier with members.
    pub members: Option<Accepts<C>>,
    /// What the target's writers need to know of it.
    pub meta: M,
}

impl<C: WireClass, M> WireType<C, M> {
    /// A carrier of a class naming one exact Rust type: `i64`, a `jlong`.
    ///
    /// Panics if the class leaves a declared type's name to fill in: that is
    /// the frontend calling the wrong constructor.
    pub fn exact(class: C, members: Option<Accepts<C>>, meta: M) -> Self {
        let rust = class.rust();
        assert!(
            !holds_placeholder(&rust),
            "{class:?} carriers are types the target declares; name one with `WireType::declared`"
        );
        WireType {
            rust,
            class,
            members,
            meta,
        }
    }

    /// A carrier of a class whose Rust type is one the target declares, named
    /// `name`: the class's type with `name` in place of `_` — `*mut Ledger`
    /// for `*mut _`.
    ///
    /// Panics if the class names one exact type, which has no name to fill in.
    pub fn declared(class: C, name: syn::Ident, members: Option<Accepts<C>>, meta: M) -> Self {
        let mut rust = class.rust();
        assert!(
            holds_placeholder(&rust),
            "{class:?} carriers are one exact type; build one with `WireType::exact`"
        );
        Name(name).visit_type_mut(&mut rust);
        WireType {
            rust,
            class,
            members,
            meta,
        }
    }

    /// The Rust type: `i64`, `*mut ledger_t`, `Stamp`, `JObject<'_>`.
    pub fn rust(&self) -> &syn::Type {
        &self.rust
    }
}

/// Whether a class's type leaves a declared type's name to fill in.
fn holds_placeholder(ty: &syn::Type) -> bool {
    struct Finds(bool);
    impl syn::visit_mut::VisitMut for Finds {
        fn visit_type_infer_mut(&mut self, _: &mut syn::TypeInfer) {
            self.0 = true;
        }
    }
    let mut finds = Finds(false);
    finds.visit_type_mut(&mut ty.clone());
    finds.0
}

/// Fills a class's `_` with the declared type's name.
struct Name(syn::Ident);

impl syn::visit_mut::VisitMut for Name {
    fn visit_type_mut(&mut self, ty: &mut syn::Type) {
        match ty {
            syn::Type::Infer(_) => {
                let name = &self.0;
                *ty = syn::parse_quote!(#name);
            }
            _ => syn::visit_mut::visit_type_mut(self, ty),
        }
    }
}

/// Which wire types a holder may hold: an aggregate's members, a wrapper's
/// parameters or its return.
///
/// The registry checks each placement once the plan has worked out what is
/// placed there, and refuses the value it would put anywhere else.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Accepts<C> {
    pub classes: Vec<C>,
}

impl<C: WireClass> Accepts<C> {
    pub fn of(classes: impl IntoIterator<Item = C>) -> Self {
        Accepts {
            classes: classes.into_iter().collect(),
        }
    }

    /// Every class of the target.
    pub fn any() -> Self {
        Self::of(C::all())
    }

    pub fn holds(&self, class: &C) -> bool {
        self.classes.contains(class)
    }
}

/// How the values a conversion rule covers cross.
///
/// It names its carriers and operations and states no type for either: a
/// codec reads its carrier and produces the source type, a `read` reads the
/// product's carrier and produces the part's, whatever carrier that part
/// resolves to. The registry works them out when it plans a value, and feeds
/// them to the writer.
// A binding holds a few dozen of these, each once, so the unboxed codecs cost
// nothing worth an indirection.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Representation<Op> {
    /// The whole value, one operation each way: a scalar, a handle, a
    /// fieldless enum. Each direction has a carrier of its own, because some
    /// values are received in a form they are not returned in — a C enum
    /// arrives as `MaybeUninit` of itself.
    Terminal {
        /// `None`: the value never crosses into Rust.
        into_rust: Option<Codec<Op>>,
        /// `None`: the value never crosses out of Rust.
        out_of_rust: Option<Codec<Op>>,
        /// How the foreign side gives a held value back without converting
        /// it — a handle's typed drop. A type output with a release exports it
        /// as a wrapper of its own.
        release: Option<Operation<Op>>,
    },
    /// The parts of a relation, carried together in one carrier, into Rust.
    Product {
        via: Via,
        /// The aggregate or object holding the parts. Must have members.
        carrier: CarrierId,
        /// One part out of the carrier, applied once per part.
        read: Operation<Op>,
    },
    /// A foreign callable, into Rust as an `impl Fn(..)`: the registry builds
    /// the closure, whose every call converts the arguments out of Rust and
    /// hands them to `invoke`.
    Callback {
        /// What holds the callable on the foreign side: a C closure struct, a
        /// JVM object. Its members are the arguments' carriers.
        carrier: CarrierId,
        /// Applied once to the carrier, where the callable enters Rust: what
        /// every call will need, such as a reference the JVM keeps alive. Its
        /// result is moved into the closure.
        capture: Operation<Op>,
        /// Applied on every call to what `capture` produced and the
        /// arguments' carriers, in order. It produces nothing.
        invoke: Operation<Op>,
        /// What a call does when converting an argument or invoking fails:
        /// the closure returns nothing, so no caller can take the failure.
        /// Reporting inside a call has no runtime context to use.
        routes: Vec<FailureRoute<Op>>,
    },
    /// A representation the target does not lower, refused by name.
    Unsupported(Unsupported),
}

/// One direction of a [`Representation::Terminal`]: the carrier, and the
/// operation between it and the source type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Codec<Op> {
    pub carrier: CarrierId,
    pub operation: Operation<Op>,
}

/// Which relation of its type a `Product` reads a value through.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Via {
    /// The struct relation: one part per field.
    Fields,
}

/// One operation, as the binding states it.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Operation<Op> {
    pub implementation: Implementation<Op>,
    /// Runtime contexts it needs, by name: `jni.env`. A function form's
    /// context parameters supply them.
    pub context: Vec<String>,
    /// Its failure category and error type, if it can fail.
    pub failure: Option<Failure>,
}

impl<Op> Operation<Op> {
    /// A standard operation, with the failure it has by definition.
    pub fn standard(op: StandardOp) -> Self {
        let failure = match &op {
            StandardOp::FromRaw
            | StandardOp::EnumIn {
                invalid: Some(_), ..
            } => Some(Failure::binding_message()),
            _ => None,
        };
        Operation {
            implementation: Implementation::Standard(op),
            context: Vec::new(),
            failure,
        }
    }

    /// One of the target's own operations, infallible until stated
    /// otherwise.
    pub fn target(op: Op) -> Self {
        Operation {
            implementation: Implementation::Target(op),
            context: Vec::new(),
            failure: None,
        }
    }

    /// The same, needing the runtime context `name`.
    pub fn context(mut self, name: impl Into<String>) -> Self {
        self.context.push(name.into());
        self
    }

    /// The same, failing in `category` with an error of type `error`.
    pub fn fails(mut self, category: FailureCategory, error: syn::Type) -> Self {
        self.failure = Some(Failure {
            category,
            error: Box::new(error),
        });
        self
    }
}

/// What actually performs an operation.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Implementation<Op> {
    Standard(StandardOp),
    Target(Op),
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
/// the carrier an address is cast to, what a value is carried as — is the
/// carrier's and the arms'.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum StandardOp {
    /// The result *is* the operand: the carrier and the source value are one
    /// Rust value. Renders nothing at all.
    Identity,
    /// Read the part's member of the aggregate the carrier is: `arg0.secs`.
    ReadMember,
    /// Hand an owned source value to the foreign side as an address:
    /// `Box::into_raw(Box::new(v)) as <carrier>`. The foreign side owns the
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
    /// that message, formatted with the value — the carrier can hold
    /// something no value of the enum names. With `bits` set the match is on
    /// the carrier's bits read as that integer type: C's `MaybeUninit<op_t>`
    /// holds whatever `int` the caller passed. The carrier must be exactly as
    /// large as `bits`, and initialized, which only the caller can promise, so
    /// the registry makes a wrapper that reads one `unsafe`.
    EnumIn {
        values: Vec<crate::target::EnumArm>,
        invalid: Option<String>,
        bits: Option<Box<syn::Type>>,
    },
}

/// How one source function is exported: everything about its wrapper that is
/// not a planned value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionForm<Op, C> {
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
    /// The wire types a wrapper parameter may be.
    pub params: Accepts<C>,
    /// The wire types the wrapper may return.
    pub ret: Accepts<C>,
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
pub enum OutputForm<Op, C, O> {
    /// One representation of a type, exposed: its carriers' Rust
    /// declarations, and its foreign declaration. The representation is also
    /// the rule at the output's root, so two outputs of one type each plan as
    /// declared.
    Type {
        representation: ReprId,
        /// The wrapper releasing a handed-out value, for a representation
        /// that has a release.
        release: Option<FunctionForm<Op, C>>,
        meta: O,
    },
    /// A source function, exported through a wrapper of this form.
    Function { form: FunctionForm<Op, C>, meta: O },
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

/// A carrier, as a binding for target `T` states one.
pub type CarrierOf<T> = WireType<<T as Target>::WireClass, <T as Target>::CarrierMeta>;
/// A representation, as a binding for target `T` states one.
pub type RepresentationOf<T> = Representation<<T as Target>::Op>;
/// A function form, as a binding for target `T` states one.
pub type FunctionFormOf<T> = FunctionForm<<T as Target>::Op, <T as Target>::WireClass>;
/// An output's form, as a binding for target `T` states one.
pub type OutputFormOf<T> =
    OutputForm<<T as Target>::Op, <T as Target>::WireClass, <T as Target>::OutputMeta>;

/// Everything planning reads from a binding.
///
/// The ids it issues are valid only in it. Every method taking an id panics if
/// the id was issued by another binding: that is a frontend bug, found where
/// the id is handed over rather than as a wrong plan.
pub struct Binding<T: Target> {
    issuer: Issuer,
    carriers: Vec<CarrierOf<T>>,
    representations: Vec<RepresentationOf<T>>,
    rules: Vec<(Scope, ReprId)>,
    outputs: Vec<(Declaration, OutputFormOf<T>)>,
}

impl<T: Target> Default for Binding<T> {
    fn default() -> Self {
        Binding {
            issuer: Issuer::fresh(),
            carriers: Vec::new(),
            representations: Vec::new(),
            rules: Vec::new(),
            outputs: Vec::new(),
        }
    }
}

impl<T: Target> Binding<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// A carrier generated Rust may use. Declaring an equal carrier again
    /// returns the same id.
    pub fn carrier(&mut self, carrier: CarrierOf<T>) -> CarrierId {
        let index = match self.carriers.iter().position(|known| *known == carrier) {
            Some(index) => index,
            None => {
                self.carriers.push(carrier);
                self.carriers.len() - 1
            }
        };
        CarrierId {
            issuer: self.issuer,
            index,
        }
    }

    /// One way a type crosses. Declaring an equal representation again
    /// returns the same id.
    ///
    /// Panics if a carrier it names was issued by another binding.
    pub fn representation(&mut self, representation: RepresentationOf<T>) -> ReprId {
        let carriers: Vec<CarrierId> = match &representation {
            Representation::Terminal {
                into_rust,
                out_of_rust,
                ..
            } => into_rust
                .iter()
                .chain(out_of_rust)
                .map(|codec| codec.carrier)
                .collect(),
            Representation::Product { carrier, .. } | Representation::Callback { carrier, .. } => {
                vec![*carrier]
            }
            Representation::Unsupported(_) => Vec::new(),
        };
        for carrier in carriers {
            self.check(carrier.issuer, "carrier");
        }
        let index = match self
            .representations
            .iter()
            .position(|known| *known == representation)
        {
            Some(index) => index,
            None => {
                self.representations.push(representation);
                self.representations.len() - 1
            }
        };
        ReprId {
            issuer: self.issuer,
            index,
        }
    }

    /// The values `scope` covers cross as `representation`.
    ///
    /// Panics if `representation`, or the output `scope` names, was issued by
    /// another binding.
    pub fn rule(&mut self, scope: Scope, representation: ReprId) {
        self.check(representation.issuer, "representation");
        if let Scope::At(output, _) = &scope {
            self.check(output.issuer, "output");
        }
        self.rules.push((scope, representation));
    }

    /// Expose `declaration` in `form`. The id is how a rule addresses a value
    /// inside the output.
    ///
    /// Panics if the representation a type form names was issued by another
    /// binding.
    pub fn output(&mut self, declaration: Declaration, form: OutputFormOf<T>) -> OutputId {
        if let OutputForm::Type { representation, .. } = &form {
            self.check(representation.issuer, "representation");
        }
        self.outputs.push((declaration, form));
        self.output_id(self.outputs.len() - 1)
    }

    /// The carrier `id` names. Panics if another binding issued `id`.
    pub fn carrier_of(&self, id: CarrierId) -> &CarrierOf<T> {
        self.check(id.issuer, "carrier");
        &self.carriers[id.index]
    }

    /// The representation `id` names. Panics if another binding issued `id`.
    pub fn representation_of(&self, id: ReprId) -> &RepresentationOf<T> {
        self.check(id.issuer, "representation");
        &self.representations[id.index]
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
    /// One line per carrier, representation, rule and output, a function
    /// form's details on the lines under its output, with the target's own
    /// vocabulary in its `Debug` form. It prints every field planning reads,
    /// which makes it the diagnostic to diff when two builds of one binding
    /// generate differently. It is not a proof that two bindings are equal:
    /// the target's classes, metadata and operations print only as much as
    /// their `Debug` tells apart.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, carrier) in self.carriers.iter().enumerate() {
            let members = match &carrier.members {
                Some(accepts) => format!("members {}", classes(&accepts.classes)),
                None => "no members".to_string(),
            };
            writeln!(
                f,
                "carrier  c{index}  {}  {}  {members}  {:?}",
                tokens(&carrier.rust),
                carrier.class.name(),
                carrier.meta
            )?;
        }
        for (index, representation) in self.representations.iter().enumerate() {
            let described = match representation {
                Representation::Terminal {
                    into_rust,
                    out_of_rust,
                    release,
                } => {
                    let codec = |codec: &Option<Codec<T::Op>>| match codec {
                        Some(codec) => format!("{} {}", codec.carrier, operation(&codec.operation)),
                        None => "-".to_string(),
                    };
                    format!(
                        "terminal  in: {}  out: {}{}",
                        codec(into_rust),
                        codec(out_of_rust),
                        match release {
                            Some(release) => format!("  release: {}", operation(release)),
                            None => String::new(),
                        }
                    )
                }
                Representation::Product { via, carrier, read } => {
                    format!("product  {carrier}  {via:?}  read: {}", operation(read))
                }
                Representation::Callback {
                    carrier,
                    capture,
                    invoke,
                    routes,
                } => format!(
                    "callback  {carrier}  capture: {}  invoke: {}  routes: [{}]",
                    operation(capture),
                    operation(invoke),
                    routes.iter().map(route).collect::<Vec<_>>().join("; ")
                ),
                Representation::Unsupported(reason) => {
                    format!("unsupported  {}: {}", reason.capability, reason.explanation)
                }
            };
            writeln!(f, "repr     r{index}  {described}")?;
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
                    representation,
                    release,
                    meta,
                } => {
                    writeln!(f, "output   {declaration}  type {representation}  {meta:?}")?;
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

/// Classes as a refusal names them: `[i64, pointer]`.
fn classes<C: WireClass>(classes: &[C]) -> String {
    let names: Vec<&str> = classes.iter().map(WireClass::name).collect();
    format!("[{}]", names.join(", "))
}

fn tokens(item: &impl quote::ToTokens) -> String {
    item.to_token_stream().to_string()
}

/// An operation on one line: what it is, the contexts it needs and the failure
/// it can raise.
fn operation<Op: std::fmt::Debug>(operation: &Operation<Op>) -> String {
    let mut text = format!("{:?}", operation.implementation);
    if !operation.context.is_empty() {
        text += &format!(" needs {}", operation.context.join(", "));
    }
    if let Some(failure) = &operation.failure {
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
/// one line, its acceptance on the next, then one line per route.
fn write_form<Op: std::fmt::Debug, C: WireClass>(
    f: &mut std::fmt::Formatter<'_>,
    label: &str,
    form: &FunctionForm<Op, C>,
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
    writeln!(
        f,
        "         {label}  params {}  ret {}",
        classes(&form.params.classes),
        classes(&form.ret.classes)
    )?;
    for one in &form.routes {
        writeln!(f, "         {label}  route {}", route(one))?;
    }
    Ok(())
}

/// A failure route on one line: its category, what reports it, and how it
/// ends.
fn route<Op: std::fmt::Debug>(route: &FailureRoute<Op>) -> String {
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
