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

use crate::{
    decl::Declaration,
    target::{FailureCategory, Target, Terminal, Unsupported},
};

/// A carrier the binding declared, valid inside the binding that issued it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CarrierId(pub(crate) usize);

/// A representation the binding declared, valid inside the binding that
/// issued it. Equal representations get one id, which is what makes it a
/// conversion's identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReprId(pub(crate) usize);

/// One of the binding's outputs: an index into the outputs it declared.
///
/// An output is a declaration *and* the form recorded with it, and the pair is
/// what tells two of them apart: one entity declared twice — `Stamp` as a data
/// class and as a handle — is two outputs, each planned, accounted for and
/// required separately.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OutputId(pub(crate) usize);

/// A Rust type generated code may hold a value in at the boundary.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WireType<C, M> {
    /// The Rust type: `i64`, `*mut ledger_t`, `Stamp`, `JObject<'_>`.
    pub rust: syn::Type,
    /// Which of the adapter's few wire types it is — what a holder's
    /// [`Accepts`] is stated in.
    pub class: C,
    /// For an aggregate, which wire types its members may be; `None` for a
    /// carrier with no members. A `Product` needs a carrier with members.
    pub members: Option<Accepts<C>>,
    /// What the target's writers need to know of it.
    pub meta: M,
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

impl<C: PartialEq> Accepts<C> {
    pub fn of(classes: impl IntoIterator<Item = C>) -> Self {
        Accepts {
            classes: classes.into_iter().collect(),
        }
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
}

impl std::fmt::Display for Step {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Step::Param(name) => write!(f, "param {name}"),
            Step::Return => write!(f, "return"),
            Step::Field(name) => write!(f, "field {name}"),
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
pub struct Binding<T: Target> {
    carriers: Vec<CarrierOf<T>>,
    representations: Vec<RepresentationOf<T>>,
    rules: Vec<(Scope, ReprId)>,
    outputs: Vec<(Declaration, OutputFormOf<T>)>,
}

impl<T: Target> Default for Binding<T> {
    fn default() -> Self {
        Binding {
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
        match self.carriers.iter().position(|known| *known == carrier) {
            Some(index) => CarrierId(index),
            None => {
                self.carriers.push(carrier);
                CarrierId(self.carriers.len() - 1)
            }
        }
    }

    /// One way a type crosses. Declaring an equal representation again
    /// returns the same id.
    pub fn representation(&mut self, representation: RepresentationOf<T>) -> ReprId {
        match self
            .representations
            .iter()
            .position(|known| *known == representation)
        {
            Some(index) => ReprId(index),
            None => {
                self.representations.push(representation);
                ReprId(self.representations.len() - 1)
            }
        }
    }

    /// The values `scope` covers cross as `representation`.
    pub fn rule(&mut self, scope: Scope, representation: ReprId) {
        self.rules.push((scope, representation));
    }

    /// Expose `declaration` in `form`. The id is how a rule addresses a value
    /// inside the output.
    pub fn output(&mut self, declaration: Declaration, form: OutputFormOf<T>) -> OutputId {
        self.outputs.push((declaration, form));
        OutputId(self.outputs.len() - 1)
    }

    pub fn carrier_of(&self, id: CarrierId) -> &CarrierOf<T> {
        &self.carriers[id.0]
    }

    pub fn representation_of(&self, id: ReprId) -> &RepresentationOf<T> {
        &self.representations[id.0]
    }

    pub fn rules(&self) -> &[(Scope, ReprId)] {
        &self.rules
    }

    pub fn outputs(&self) -> &[(Declaration, OutputFormOf<T>)] {
        &self.outputs
    }

    /// The form recorded with one output.
    pub fn form_of(&self, id: OutputId) -> &OutputFormOf<T> {
        &self.outputs[id.0].1
    }
}

impl<T: Target> std::fmt::Display for Binding<T> {
    /// One line per carrier, representation, rule and output, with the
    /// target's own vocabulary in its `Debug` form: the whole of what planning
    /// reads, which is what to diff when two builds of one binding generate
    /// differently.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use quote::ToTokens;
        for (index, carrier) in self.carriers.iter().enumerate() {
            writeln!(
                f,
                "carrier  c{index}  {}  {:?}  {:?}",
                carrier.rust.to_token_stream(),
                carrier.class,
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
                        Some(codec) => {
                            format!("c{} {:?}", codec.carrier.0, codec.operation.implementation)
                        }
                        None => "-".to_string(),
                    };
                    format!(
                        "terminal  in: {}  out: {}{}",
                        codec(into_rust),
                        codec(out_of_rust),
                        match release {
                            Some(release) => format!("  release: {:?}", release.implementation),
                            None => String::new(),
                        }
                    )
                }
                Representation::Product { via, carrier, read } => format!(
                    "product  c{}  {via:?}  read: {:?}",
                    carrier.0, read.implementation
                ),
                Representation::Unsupported(reason) => {
                    format!("unsupported  {}", reason.capability)
                }
            };
            writeln!(f, "repr     r{index}  {described}")?;
        }
        for (scope, representation) in &self.rules {
            let scope = match scope {
                Scope::Type(key) => format!("type {}", key.as_str()),
                Scope::At(output, path) => format!("at {} {path}", self.outputs[output.0].0),
            };
            writeln!(f, "rule     {scope}  r{}", representation.0)?;
        }
        for (declaration, form) in &self.outputs {
            let described = match form {
                OutputForm::Type {
                    representation,
                    meta,
                    ..
                } => format!("type r{}  {meta:?}", representation.0),
                OutputForm::Function { form, meta } => {
                    format!("function {:?} {}  {meta:?}", form.abi, form.symbol)
                }
                OutputForm::Unsupported(reason) => format!("unsupported {}", reason.capability),
            };
            writeln!(f, "output   {declaration}  {described}")?;
        }
        Ok(())
    }
}
